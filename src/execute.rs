use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response, Uint128, BankMsg
};
use crate::error::ContractError;
use crate::msg::{MemberStatus, RoundState};
use crate::state::{
    Member, Round, Distribution,
    get_config, get_tontine_state, get_member, get_current_round,
    get_accumulated_fees, validate_member_address, validate_amount, members, ROUNDS, DISTRIBUTIONS,
    ACCUMULATED_FEES, TONTINE_STATE
};






pub struct ExecuteHandler;

impl ExecuteHandler {
    // Member Management Functions
    
    pub fn register_member(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin can register members
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can register members".to_string() 
            });
        }

        // Validate address
        validate_member_address(&address)?;
        let member_addr = deps.api.addr_validate(&address)?;

        // Check if member already exists
        if members().has(deps.storage, member_addr.as_str()) {
            return Err(ContractError::MemberAlreadyExists { address });
        }

        // Create new member
        let member = Member {
            address: member_addr.clone(),
            status: MemberStatus::Active,
            balance: Uint128::zero(),
            penalties: Uint128::zero(),
            last_contribution: None,
            is_late: false,
            registration_time: env.block.time,
        };

        // Save member
        members().save(deps.storage, member_addr.as_str(), &member)?;

        // Create response
        let response = Response::new()
            .add_attribute("method", "register_member")
            .add_attribute("member", address)
            .add_attribute("registration_time", env.block.time.to_string());

        Ok(response)
    }

    pub fn remove_member(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let state = get_tontine_state(deps.storage)?;
        
        // Only admin can remove members
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can remove members".to_string() 
            });
        }

        // Cannot remove members during active round
        if state.is_active && state.current_round > 0 {
            return Err(ContractError::CannotReplaceDuringActiveRound);
        }

        // Check if member exists
        if !members().has(deps.storage, address.as_str()) {
            return Err(ContractError::MemberNotFound { address: address.clone() });
        }

        // Get member and check if they have pending penalties
        let member = get_member(deps.storage, &deps.api.addr_validate(&address)?)?;
        if member.penalties > Uint128::zero() {
            return Err(ContractError::MemberHasPenalties);
        }

        // Remove member
        members().remove(deps.storage, address.as_str());

        let response = Response::new()
            .add_attribute("method", "remove_member")
            .add_attribute("member", address)
            .add_attribute("removal_time", env.block.time.to_string());

        Ok(response)
    }

    pub fn replace_member(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        old_address: String,
        new_address: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let state = get_tontine_state(deps.storage)?;
        
        // Only admin can replace members
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can replace members".to_string() 
            });
        }

        // Cannot replace members during active round
        if state.is_active && state.current_round > 0 {
            return Err(ContractError::CannotReplaceDuringActiveRound);
        }

        // Validate addresses
        validate_member_address(&old_address)?;
        validate_member_address(&new_address)?;
        
        let old_addr = deps.api.addr_validate(&old_address)?;
        let new_addr = deps.api.addr_validate(&new_address)?;

        // Check if old member exists
        if !members().has(deps.storage, old_address.as_str()) {
            return Err(ContractError::MemberNotFound { address: old_address });
        }

        // Check if new member already exists
        if members().has(deps.storage, new_address.as_str()) {
            return Err(ContractError::MemberAlreadyExists { address: new_address });
        }

        // Get old member
        let mut old_member = get_member(deps.storage, &old_addr)?;
        
        // Check if old member has pending penalties
        if old_member.penalties > Uint128::zero() {
            return Err(ContractError::MemberHasPenalties);
        }

        // Create new member with old member's balance
        let new_member = Member {
            address: new_addr.clone(),
            status: MemberStatus::Active,
            balance: old_member.balance,
            penalties: Uint128::zero(),
            last_contribution: old_member.last_contribution,
            is_late: false,
            registration_time: env.block.time,
        };

        // Update old member status
        old_member.status = MemberStatus::Replaced;

        // Save both members
        members().save(deps.storage, old_address.as_str(), &old_member)?;
        members().save(deps.storage, new_address.as_str(), &new_member)?;



        let response = Response::new()
            .add_attribute("method", "replace_member")
            .add_attribute("old_member", old_address)
            .add_attribute("new_member", new_address)
            .add_attribute("replacement_time", env.block.time.to_string());

        Ok(response)
    }

    // Tontine Control Functions

    pub fn start_tontine(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let mut state = get_tontine_state(deps.storage)?;
        
        // Only admin can start tontine
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can start tontine".to_string() 
            });
        }

        // Check if tontine is already started
        if state.is_active {
            return Err(ContractError::TontineAlreadyStarted);
        }

        // Check if tontine is finished
        if state.is_finished {
            return Err(ContractError::TontineAlreadyFinished);
        }

        // Get member count - since Map doesn't have range(), we need to check if any members exist
        // We'll check if at least one member is registered by looking at the beneficiaries
        let mut member_count = 0;
        for beneficiary in &config.beneficiaries {
            if members().has(deps.storage, beneficiary.as_str()) {
                member_count += 1;
            }
        }
        
        if member_count == 0 {
            return Err(ContractError::InvalidMemberManagement { 
                msg: "No members registered".to_string() 
            });
        }
        
        // Ensure first beneficiary is registered
        let first_beneficiary = &config.beneficiaries[0];
        if !members().has(deps.storage, first_beneficiary.as_str()) {
            return Err(ContractError::InvalidMemberManagement { 
                msg: "First beneficiary is not registered as a member".to_string() 
            });
        }

        // Start tontine
        state.is_active = true;
        state.start_time = Some(env.block.time);
        state.current_round = 1;
        state.total_rounds = member_count as u64;

        // Create first round
        let first_round = Round {
            round_number: 1,
            state: RoundState::Active,
            balance: Uint128::zero(),
            beneficiary: first_beneficiary.clone(),
            deadline: env.block.time.plus_seconds(config.round_frequency),
            deposits: vec![],
            is_distributed: false,
            distribution_time: None,
        };

        // Save state and round
        TONTINE_STATE.save(deps.storage, &state)?;
        ROUNDS.save(deps.storage, 1, &first_round)?;

        let response = Response::new()
            .add_attribute("method", "start_tontine")
            .add_attribute("start_time", env.block.time.to_string())
            .add_attribute("current_round", "1")
            .add_attribute("total_rounds", state.total_rounds.to_string())
            .add_attribute("first_beneficiary", first_beneficiary.to_string());

        Ok(response)
    }

    pub fn pause_tontine(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let mut state = get_tontine_state(deps.storage)?;
        
        // Only admin can pause tontine
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can pause tontine".to_string() 
            });
        }

        // Check if tontine is active
        if !state.is_active {
            return Err(ContractError::TontineNotStarted);
        }

        // Pause tontine
        state.is_paused = true;
        TONTINE_STATE.save(deps.storage, &state)?;

        let response = Response::new()
            .add_attribute("method", "pause_tontine")
            .add_attribute("pause_time", env.block.time.to_string());

        Ok(response)
    }

    pub fn resume_tontine(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let mut state = get_tontine_state(deps.storage)?;
        
        // Only admin can resume tontine
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can resume tontine".to_string() 
            });
        }

        // Check if tontine is paused
        if !state.is_paused {
            return Err(ContractError::InvalidStateUpdate { 
                msg: "Tontine is not paused".to_string() 
            });
        }

        // Resume tontine
        state.is_paused = false;
        TONTINE_STATE.save(deps.storage, &state)?;

        let response = Response::new()
            .add_attribute("method", "resume_tontine")
            .add_attribute("resume_time", env.block.time.to_string());

        Ok(response)
    }

    pub fn close_early(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        reason: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let mut state = get_tontine_state(deps.storage)?;
        
        // Only admin can close tontine early
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can close tontine early".to_string() 
            });
        }

        // Check if tontine is active
        if !state.is_active {
            return Err(ContractError::TontineNotStarted);
        }

        // Check if tontine is already finished
        if state.is_finished {
            return Err(ContractError::TontineAlreadyFinished);
        }

        // Close tontine early
        state.is_active = false;
        state.is_finished = true;
        TONTINE_STATE.save(deps.storage, &state)?;

        let response = Response::new()
            .add_attribute("method", "close_early")
            .add_attribute("reason", reason)
            .add_attribute("close_time", env.block.time.to_string());

        Ok(response)
    }

    // Round Operations

    pub fn deposit_contribution(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let state = get_tontine_state(deps.storage)?;
        
        // Check if tontine is active and not paused
        if !state.is_active || state.is_paused {
            return Err(ContractError::TontineNotStarted);
        }

        // Check if there's an active round
        if state.current_round == 0 {
            return Err(ContractError::NoActiveRound);
        }

        // Get current round
        let mut round = get_current_round(deps.storage)?;
        
        // Check if round is active
        if round.state != RoundState::Active {
            return Err(ContractError::RoundNotActive);
        }

        // Check if member exists and is active
        let member = get_member(deps.storage, &info.sender)?;
        if member.status != MemberStatus::Active {
            return Err(ContractError::InvalidMemberState { 
                state: format!("{:?}", member.status) 
            });
        }

        // Check if member already contributed to this round
        if round.deposits.iter().any(|d| d.member == info.sender) {
            return Err(ContractError::MemberAlreadyContributed);
        }

        // Check if member has pending penalties
        if member.penalties > Uint128::zero() {
            return Err(ContractError::MemberHasPenalties);
        }

        // Check if deadline has passed
        let is_late = env.block.time > round.deadline;

        // Create deposit
        let deposit = crate::state::Deposit {
            member: info.sender.clone(),
            amount: config.contribution_amount,
            timestamp: env.block.time,
            is_late,
        };

        // Add deposit to round
        round.deposits.push(deposit);
        round.balance += config.contribution_amount;

        // Update member's last contribution
        let mut updated_member = member;
        updated_member.last_contribution = Some(env.block.time);
        updated_member.is_late = is_late;

        // Save round and member
        ROUNDS.save(deps.storage, state.current_round, &round)?;
        members().save(deps.storage, info.sender.as_str(), &updated_member)?;

        let response = Response::new()
            .add_attribute("method", "deposit_contribution")
            .add_attribute("member", info.sender.to_string())
            .add_attribute("amount", config.contribution_amount.to_string())
            .add_attribute("round", state.current_round.to_string())
            .add_attribute("is_late", is_late.to_string());

        Ok(response)
    }

    pub fn distribute_to_beneficiary(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let state = get_tontine_state(deps.storage)?;
        
        // Only admin or arbitrator can distribute
        if info.sender != config.admin && info.sender != config.arbitrator {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin or arbitrator can distribute".to_string() 
            });
        }

        // Check if tontine is active
        if !state.is_active || state.is_paused {
            return Err(ContractError::TontineNotStarted);
        }

        // Get current round
        let mut round = get_current_round(deps.storage)?;
        
        // Check if round is active
        if round.state != RoundState::Active {
            return Err(ContractError::RoundNotActive);
        }

        // Check if round deadline has passed
        if env.block.time <= round.deadline {
            return Err(ContractError::RoundDeadlineNotReached);
        }

        // Check if round has been distributed
        if round.is_distributed {
            return Err(ContractError::InvalidDistribution { 
                msg: "Round already distributed".to_string() 
            });
        }

        // Calculate distribution amount (total balance minus fees)
        let total_fees = config.protocol_fees * Uint128::from(round.deposits.len() as u32);
        let distribution_amount = round.balance - total_fees;

        // Update round state
        round.state = RoundState::Distributed;
        round.is_distributed = true;
        round.distribution_time = Some(env.block.time);

        // Save round
        ROUNDS.save(deps.storage, state.current_round, &round)?;

        // Create distribution record
        let distribution = Distribution {
            round: state.current_round,
            beneficiary: round.beneficiary.clone(),
            amount: distribution_amount,
            timestamp: env.block.time,
        };

        // Save distribution
        DISTRIBUTIONS.save(deps.storage, state.current_round, &distribution)?;

        // Update accumulated fees
        let mut accumulated_fees = get_accumulated_fees(deps.storage)?;
        accumulated_fees += total_fees;
        ACCUMULATED_FEES.save(deps.storage, &accumulated_fees)?;

        // Create bank message for distribution
        let bank_msg = BankMsg::Send {
            to_address: round.beneficiary.to_string(),
            amount: vec![cosmwasm_std::Coin {
                denom: config.token_denom.clone(),
                amount: distribution_amount,
            }],
        };

        let response = Response::new()
            .add_message(bank_msg)
            .add_attribute("method", "distribute_to_beneficiary")
            .add_attribute("round", state.current_round.to_string())
            .add_attribute("beneficiary", round.beneficiary.to_string())
            .add_attribute("amount", distribution_amount.to_string())
            .add_attribute("fees", total_fees.to_string());

        Ok(response)
    }

    // Additional execute functions would be implemented here...
    // For brevity, I'm showing the key ones above

    pub fn advance_payment(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        beneficiary: String,
        discount: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin can make advance payments
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can make advance payments".to_string() 
            });
        }

        // Validate beneficiary address
        let beneficiary_addr = deps.api.addr_validate(&beneficiary)?;
        
        // Check if beneficiary is in the list
        if !config.beneficiaries.contains(&beneficiary_addr) {
            return Err(ContractError::InvalidMemberManagement { 
                msg: "Beneficiary not in beneficiaries list".to_string() 
            });
        }

        // Parse discount (percentage as string)
        let discount_percent = discount.parse::<u64>().map_err(|_| ContractError::InvalidAmount { 
            msg: "Invalid discount format".to_string() 
        })?;

        if discount_percent > 100 {
            return Err(ContractError::InvalidAmount { 
                msg: "Discount cannot exceed 100%".to_string() 
            });
        }

        let response = Response::new()
            .add_attribute("method", "advance_payment")
            .add_attribute("beneficiary", beneficiary)
            .add_attribute("discount", discount);

        Ok(response)
    }

    pub fn declare_late(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        member: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin or arbitrator can declare members late
        if info.sender != config.admin && info.sender != config.arbitrator {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin or arbitrator can declare members late".to_string() 
            });
        }

        // Validate member address
        let member_addr = deps.api.addr_validate(&member)?;
        
        // Check if member exists
        if !members().has(deps.storage, member_addr.as_str()) {
            return Err(ContractError::MemberNotFound { address: member.clone() });
        }

        let response = Response::new()
            .add_attribute("method", "declare_late")
            .add_attribute("member", member);

        Ok(response)
    }

    pub fn apply_penalty(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        member: String,
        amount: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin or arbitrator can apply penalties
        if info.sender != config.admin && info.sender != config.arbitrator {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin or arbitrator can apply penalties".to_string() 
            });
        }

        // Validate member address
        let member_addr = deps.api.addr_validate(&member)?;
        
        // Check if member exists
        if !members().has(deps.storage, member_addr.as_str()) {
            return Err(ContractError::MemberNotFound { address: member.clone() });
        }

        // Parse penalty amount
        let _penalty_amount = validate_amount(&amount)?;

        let response = Response::new()
            .add_attribute("method", "apply_penalty")
            .add_attribute("member", member)
            .add_attribute("amount", amount);

        Ok(response)
    }

    pub fn pay_penalty(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        member: String,
    ) -> Result<Response, ContractError> {
        let _config = get_config(deps.storage)?;
        
        // Only the member themselves can pay their penalty
        let member_addr = deps.api.addr_validate(&member)?;
        if info.sender != member_addr {
            return Err(ContractError::Unauthorized { 
                msg: "Only the member can pay their own penalty".to_string() 
            });
        }

        // Check if member exists
        if !members().has(deps.storage, member_addr.as_str()) {
            return Err(ContractError::MemberNotFound { address: member.clone() });
        }

        let response = Response::new()
            .add_attribute("method", "pay_penalty")
            .add_attribute("member", member);

        Ok(response)
    }

    pub fn withdraw_fees(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin can withdraw fees
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can withdraw fees".to_string() 
            });
        }

        let response = Response::new()
            .add_attribute("method", "withdraw_fees");

        Ok(response)
    }

    pub fn collect_protocol_fees(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin can collect protocol fees
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can collect protocol fees".to_string() 
            });
        }

        let response = Response::new()
            .add_attribute("method", "collect_protocol_fees");

        Ok(response)
    }

    pub fn resolve_dispute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        member: String,
        resolution: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin can resolve disputes
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can resolve disputes".to_string() 
            });
        }

        // Validate member address
        let member_addr = deps.api.addr_validate(&member)?;
        
        // Check if member exists
        if !members().has(deps.storage, member_addr.as_str()) {
            return Err(ContractError::MemberNotFound { address: member.clone() });
        }

        let response = Response::new()
            .add_attribute("method", "resolve_dispute")
            .add_attribute("member", member)
            .add_attribute("resolution", resolution);

        Ok(response)
    }

    pub fn arbitrate_dispute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        member: String,
        decision: String,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only arbitrator can arbitrate disputes
        if info.sender != config.arbitrator {
            return Err(ContractError::Unauthorized { 
                msg: "Only arbitrator can arbitrate disputes".to_string() 
            });
        }

        // Validate member address
        let member_addr = deps.api.addr_validate(&member)?;
        
        // Check if member exists
        if !members().has(deps.storage, member_addr.as_str()) {
            return Err(ContractError::MemberNotFound { address: member.clone() });
        }

        let response = Response::new()
            .add_attribute("method", "arbitrate_dispute")
            .add_attribute("member", member)
            .add_attribute("decision", decision);

        Ok(response)
    }

    pub fn finalize_tontine(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        let mut state = get_tontine_state(deps.storage)?;
        
        // Only admin can finalize tontine
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can finalize tontine".to_string() 
            });
        }

        // Check if tontine is active
        if !state.is_active {
            return Err(ContractError::TontineNotStarted);
        }

        // Check if all rounds are completed
        if state.current_round < state.total_rounds {
            return Err(ContractError::InvalidStateUpdate { 
                msg: "Cannot finalize before all rounds are completed".to_string() 
            });
        }

        // Finalize tontine
        state.is_finished = true;
        state.is_active = false;
        TONTINE_STATE.save(deps.storage, &state)?;

        let response = Response::new()
            .add_attribute("method", "finalize_tontine")
            .add_attribute("finalization_time", env.block.time.to_string());

        Ok(response)
    }

    pub fn migrate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        new_code_id: u64,
    ) -> Result<Response, ContractError> {
        let config = get_config(deps.storage)?;
        
        // Only admin can migrate contract
        if info.sender != config.admin {
            return Err(ContractError::Unauthorized { 
                msg: "Only admin can migrate contract".to_string() 
            });
        }

        let response = Response::new()
            .add_attribute("method", "migrate")
            .add_attribute("new_code_id", new_code_id.to_string());

        Ok(response)
    }
}
