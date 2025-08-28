use cosmwasm_std::{
    Binary, Deps, StdError, StdResult, Uint128
};
use cosmwasm_std::to_json_binary as to_binary;
use crate::msg::{
    QueryMsg, ConfigResponse, MemberResponse, TontineStateResponse,
    DistributionHistoryResponse, PenaltyHistoryResponse, DepositHistoryResponse,
    EscrowStateResponse, DisputeStateResponse, StatisticsResponse, RoundState,
    DistributionResponse, PenaltyResponse, DepositResponse
};
use crate::state::{
    get_config, get_tontine_state, get_escrow_state, get_member, get_round,
    get_current_round, get_accumulated_fees, members, DISTRIBUTIONS, PENALTIES
};

pub struct QueryHandler;

impl QueryHandler {
    pub fn handle_query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::GetConfig {} => Self::get_config(deps),
            QueryMsg::GetAdmin {} => Self::get_admin(deps),
            QueryMsg::GetArbitrator {} => Self::get_arbitrator(deps),
            QueryMsg::GetMembers {} => Self::get_members(deps),
            QueryMsg::GetMember { address } => Self::get_member(deps, address),
            QueryMsg::GetMemberStatus { address } => Self::get_member_status(deps, address),
            QueryMsg::GetMemberBalance { address } => Self::get_member_balance(deps, address),
            QueryMsg::GetMemberPenalties { address } => Self::get_member_penalties(deps, address),
            QueryMsg::GetCurrentRound {} => Self::get_current_round(deps),
            QueryMsg::GetRoundInfo { round } => Self::get_round_info(deps, round),
            QueryMsg::GetRoundDeposits { round } => Self::get_round_deposits(deps, round),
            QueryMsg::GetRoundState { round } => Self::get_round_state(deps, round),
            QueryMsg::GetTontineBalance {} => Self::get_tontine_balance(deps),
            QueryMsg::GetRoundBalance { round } => Self::get_round_balance(deps, round),
            QueryMsg::GetAccumulatedFees {} => Self::get_accumulated_fees(deps),
            QueryMsg::GetPendingPenalties {} => Self::get_pending_penalties(deps),
            QueryMsg::GetCurrentBeneficiary {} => Self::get_current_beneficiary(deps),
            QueryMsg::GetNextBeneficiary {} => Self::get_next_beneficiary(deps),
            QueryMsg::GetBeneficiariesList {} => Self::get_beneficiaries_list(deps),
            QueryMsg::GetBeneficiarySchedule {} => Self::get_beneficiary_schedule(deps),
            QueryMsg::GetRoundDeadline { round } => Self::get_round_deadline(deps, round),
            QueryMsg::GetTimeGuards {} => Self::get_time_guards(deps),
            QueryMsg::GetRoundFrequency {} => Self::get_round_frequency(deps),
            QueryMsg::GetDistributionHistory {} => Self::get_distribution_history(deps),
            QueryMsg::GetPenaltyHistory {} => Self::get_penalty_history(deps),
            QueryMsg::GetDepositHistory {} => Self::get_deposit_history(deps),
            QueryMsg::GetTontineState {} => Self::get_tontine_state(deps),
            QueryMsg::GetEscrowState {} => Self::get_escrow_state(deps),
            QueryMsg::GetDisputeState {} => Self::get_dispute_state(deps),
            QueryMsg::GetMemberCount {} => Self::get_member_count(deps),
            QueryMsg::GetTotalContributions {} => Self::get_total_contributions(deps),
            QueryMsg::GetTotalDistributions {} => Self::get_total_distributions(deps),
            QueryMsg::GetTotalPenalties {} => Self::get_total_penalties(deps),
            QueryMsg::GetTotalFees {} => Self::get_total_fees(deps),
            QueryMsg::GetStatistics {} => Self::get_statistics(deps),
        }
    }

    // Configuration queries
    pub fn get_config(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        let response = ConfigResponse {
            admin: config.admin.to_string(),
            token_denom: config.token_denom,
            contribution_amount: config.contribution_amount.to_string(),
            round_frequency: config.round_frequency,
            beneficiaries: config.beneficiaries.iter().map(|addr| addr.to_string()).collect(),
            late_penalty: config.late_penalty.to_string(),
            protocol_fees: config.protocol_fees.to_string(),
            arbitrator: config.arbitrator.to_string(),
            time_guards: config.time_guards,
            is_active: state.is_active,
            is_paused: state.is_paused,
            is_finished: state.is_finished,
        };
        to_binary(&response)
    }

    pub fn get_admin(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&config.admin.to_string())
    }

    pub fn get_arbitrator(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&config.arbitrator.to_string())
    }

    // Member queries
    pub fn get_members(deps: Deps) -> StdResult<Binary> {
        let members_list: StdResult<Vec<MemberResponse>> = members()
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .map(|item| {
                let (addr, member) = item?;
                
                // Calculate actual balance from rounds
                let mut balance = Uint128::zero();
                let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
                
                for round_num in 1..=state.total_rounds {
                    if let Ok(round) = get_round(deps.storage, round_num) {
                        // Check if member contributed to this round
                        if round.deposits.iter().any(|d| d.member == member.address) {
                            balance += round.balance;
                        }
                    }
                }
                
                Ok(MemberResponse {
                    address: addr.to_string(),
                    status: member.status.clone(),
                    balance: balance.to_string(),
                    penalties: member.penalties.to_string(),
                    last_contribution: member.last_contribution,
                    is_late: member.is_late,
                })
            })
            .collect();
        
        to_binary(&members_list?)
    }

    pub fn get_member(deps: Deps, address: String) -> StdResult<Binary> {
        let validated_addr = deps.api.addr_validate(&address)?;
        let member = get_member(deps.storage, &validated_addr).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        let response = MemberResponse {
            address: member.address.to_string(),
            status: member.status.clone(),
            balance: "0".to_string(), // Placeholder - would calculate actual balance
            penalties: member.penalties.to_string(),
            last_contribution: member.last_contribution,
            is_late: member.is_late,
        };
        
        to_binary(&response)
    }

    pub fn get_member_status(deps: Deps, address: String) -> StdResult<Binary> {
        let validated_addr = deps.api.addr_validate(&address)?;
        let member = get_member(deps.storage, &validated_addr).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&member.status)
    }

    pub fn get_member_balance(deps: Deps, address: String) -> StdResult<Binary> {
        let validated_addr = deps.api.addr_validate(&address)?;
        let member = get_member(deps.storage, &validated_addr).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        // Calculate actual balance from rounds
        let mut balance = Uint128::zero();
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        for round_num in 1..=state.total_rounds {
            if let Ok(round) = get_round(deps.storage, round_num) {
                // Check if member contributed to this round
                if round.deposits.iter().any(|d| d.member == member.address) {
                    balance += round.balance;
                }
            }
        }
        
        to_binary(&balance.to_string())
    }

    pub fn get_member_penalties(deps: Deps, address: String) -> StdResult<Binary> {
        let validated_addr = deps.api.addr_validate(&address)?;
        let member = get_member(deps.storage, &validated_addr).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&member.penalties.to_string())
    }

    // Round queries
    pub fn get_current_round(deps: Deps) -> StdResult<Binary> {
        let round = get_current_round(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&round)
    }

    pub fn get_round_info(deps: Deps, round: u64) -> StdResult<Binary> {
        let round_data = get_round(deps.storage, round).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&round_data)
    }

    pub fn get_round_deposits(deps: Deps, round: u64) -> StdResult<Binary> {
        let round_data = get_round(deps.storage, round).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&round_data.deposits)
    }

    pub fn get_round_state(deps: Deps, round: u64) -> StdResult<Binary> {
        let round_data = get_round(deps.storage, round).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&round_data.state)
    }

    pub fn get_tontine_balance(deps: Deps) -> StdResult<Binary> {
        // Calculate total balance from all rounds
        let mut total_balance = Uint128::zero();
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        for round_num in 1..=state.total_rounds {
            if let Ok(round) = get_round(deps.storage, round_num) {
                total_balance += round.balance;
            }
        }
        
        to_binary(&total_balance.to_string())
    }

    pub fn get_round_balance(deps: Deps, round: u64) -> StdResult<Binary> {
        let round_data = get_round(deps.storage, round).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&round_data.balance.to_string())
    }

    pub fn get_accumulated_fees(deps: Deps) -> StdResult<Binary> {
        let fees = get_accumulated_fees(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&fees.to_string())
    }

    pub fn get_pending_penalties(deps: Deps) -> StdResult<Binary> {
        let mut pending_penalties = Uint128::zero();
        
        // Sum up all unpaid penalties
        let penalties: StdResult<Vec<_>> = PENALTIES
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();
        
        if let Ok(penalties_list) = penalties {
            for (_, penalty) in penalties_list {
                if !penalty.is_paid {
                    pending_penalties += penalty.amount;
                }
            }
        }
        
        to_binary(&pending_penalties.to_string())
    }

    // Beneficiary queries
    pub fn get_current_beneficiary(deps: Deps) -> StdResult<Binary> {
        let round = get_current_round(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&round.beneficiary.to_string())
    }

    pub fn get_next_beneficiary(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        if state.current_round >= state.total_rounds {
            return to_binary(&"No next beneficiary - tontine completed");
        }
        
        let next_round = state.current_round + 1;
        if next_round <= config.beneficiaries.len() as u64 {
            let next_beneficiary = &config.beneficiaries[(next_round - 1) as usize];
            to_binary(&next_beneficiary.to_string())
        } else {
            to_binary(&"Invalid beneficiary index")
        }
    }

    pub fn get_beneficiaries_list(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        let beneficiaries: Vec<String> = config.beneficiaries.iter()
            .map(|addr| addr.to_string())
            .collect();
        to_binary(&beneficiaries)
    }

    pub fn get_beneficiary_schedule(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        let _state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        let schedule: Vec<(u64, String)> = config.beneficiaries.iter()
            .enumerate()
            .map(|(i, addr)| (i as u64 + 1, addr.to_string()))
            .collect();
        
        to_binary(&schedule)
    }

    // Time queries
    pub fn get_round_deadline(deps: Deps, round: u64) -> StdResult<Binary> {
        let round_data = get_round(deps.storage, round).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&round_data.deadline)
    }

    pub fn get_time_guards(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&config.time_guards)
    }

    pub fn get_round_frequency(deps: Deps) -> StdResult<Binary> {
        let config = get_config(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&config.round_frequency)
    }

    // Historical queries
    pub fn get_distribution_history(deps: Deps) -> StdResult<Binary> {
        let mut distributions = Vec::new();
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        for round_num in 1..=state.total_rounds {
            if let Ok(distribution) = DISTRIBUTIONS.load(deps.storage, round_num) {
                let distribution_response = DistributionResponse {
                    round: distribution.round,
                    beneficiary: distribution.beneficiary.to_string(),
                    amount: distribution.amount.to_string(),
                    timestamp: distribution.timestamp,
                };
                distributions.push(distribution_response);
            }
        }
        
        to_binary(&DistributionHistoryResponse { distributions })
    }

    pub fn get_penalty_history(deps: Deps) -> StdResult<Binary> {
        let penalties: StdResult<Vec<_>> = PENALTIES
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();
        
        let penalties_list = penalties.map_err(|e| StdError::generic_err(e.to_string()))?;
        let penalties_data: Vec<PenaltyResponse> = penalties_list.into_iter().map(|(_, penalty)| PenaltyResponse {
            member: penalty.member.to_string(),
            amount: penalty.amount.to_string(),
            reason: penalty.reason,
            timestamp: penalty.timestamp,
            is_paid: penalty.is_paid,
        }).collect();
        
        to_binary(&PenaltyHistoryResponse { penalties: penalties_data })
    }

    pub fn get_deposit_history(deps: Deps) -> StdResult<Binary> {
        let mut all_deposits = Vec::new();
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        for round_num in 1..=state.total_rounds {
            if let Ok(round) = get_round(deps.storage, round_num) {
                for deposit in &round.deposits {
                    let deposit_response = DepositResponse {
                        member: deposit.member.to_string(),
                        amount: deposit.amount.to_string(),
                        timestamp: deposit.timestamp,
                        is_late: deposit.is_late,
                    };
                    all_deposits.push(deposit_response);
                }
            }
        }
        
        to_binary(&DepositHistoryResponse { deposits: all_deposits })
    }

    pub fn get_tontine_state(deps: Deps) -> StdResult<Binary> {
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        // Calculate actual member count
        let member_count = members()
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .count() as u64;
        
        // Calculate actual total balance from all rounds
        let mut total_balance = Uint128::zero();
        for round_num in 1..=state.total_rounds {
            if let Ok(round) = get_round(deps.storage, round_num) {
                total_balance += round.balance;
            }
        }
        
        // Get accumulated fees
        let total_fees = get_accumulated_fees(deps.storage)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        
        // Calculate total penalties
        let mut total_penalties = Uint128::zero();
        let penalties: StdResult<Vec<_>> = PENALTIES
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();
        
        if let Ok(penalties_list) = penalties {
            for (_, penalty) in penalties_list {
                total_penalties += penalty.amount;
            }
        }
        
        let response = TontineStateResponse {
            current_round: state.current_round,
            total_rounds: state.total_rounds,
            total_balance: total_balance.to_string(),
            total_fees: total_fees.to_string(),
            total_penalties: total_penalties.to_string(),
            member_count,
            is_active: state.is_active,
            is_paused: state.is_paused,
            is_finished: state.is_finished,
        };
        
        to_binary(&response)
    }

    pub fn get_escrow_state(deps: Deps) -> StdResult<Binary> {
        let escrow = get_escrow_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        let response = EscrowStateResponse {
            is_locked: escrow.is_locked,
            locked_amount: escrow.locked_amount.to_string(),
            lock_reason: escrow.lock_reason,
            lock_timestamp: escrow.lock_timestamp,
        };
        
        to_binary(&response)
    }

    pub fn get_dispute_state(_deps: Deps) -> StdResult<Binary> {
        to_binary(&DisputeStateResponse { active_disputes: vec![] }) // Placeholder
    }

    // Statistics queries
    pub fn get_member_count(deps: Deps) -> StdResult<Binary> {
        let member_count = members()
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .count();
        to_binary(&(member_count as u64))
    }

    pub fn get_total_contributions(deps: Deps) -> StdResult<Binary> {
        let mut total = Uint128::zero();
        
        // Get tontine state to know how many rounds exist
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        // Sum up all round balances
        for round_num in 1..=state.total_rounds {
            if let Ok(round) = get_round(deps.storage, round_num) {
                total += round.balance;
            }
        }
        
        to_binary(&total.to_string())
    }

    pub fn get_total_distributions(deps: Deps) -> StdResult<Binary> {
        let mut total = Uint128::zero();
        
        // Get tontine state to know how many rounds exist
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        // Sum up all distributions
        for round_num in 1..=state.total_rounds {
            if let Ok(distribution) = DISTRIBUTIONS.load(deps.storage, round_num) {
                total += distribution.amount;
            }
        }
        
        to_binary(&total.to_string())
    }

    pub fn get_total_penalties(deps: Deps) -> StdResult<Binary> {
        let mut total = Uint128::zero();
        
        // Sum up all penalties
        let penalties: StdResult<Vec<_>> = PENALTIES
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();
        
        if let Ok(penalties_list) = penalties {
            for (_, penalty) in penalties_list {
                total += penalty.amount;
            }
        }
        
        to_binary(&total.to_string())
    }

    pub fn get_total_fees(deps: Deps) -> StdResult<Binary> {
        let fees = get_accumulated_fees(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        to_binary(&fees.to_string())
    }

    pub fn get_statistics(deps: Deps) -> StdResult<Binary> {
        let member_count = members()
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .count() as u64;
        
        let mut total_contributions = Uint128::zero();
        let mut total_distributions = Uint128::zero();
        let mut active_rounds = 0u64;
        let mut completed_rounds = 0u64;
        
        // Get tontine state
        let state = get_tontine_state(deps.storage).map_err(|e| StdError::generic_err(e.to_string()))?;
        
        // Calculate round statistics
        for round_num in 1..=state.total_rounds {
            if let Ok(round) = get_round(deps.storage, round_num) {
                total_contributions += round.balance;
                
                match round.state {
                    RoundState::Active => active_rounds += 1,
                    RoundState::Distributed => completed_rounds += 1,
                    _ => {}
                }
            }
        }
        
        // Calculate total distributions
        for round_num in 1..=state.total_rounds {
            if let Ok(distribution) = DISTRIBUTIONS.load(deps.storage, round_num) {
                total_distributions += distribution.amount;
            }
        }
        
        // Calculate total penalties
        let mut total_penalties = Uint128::zero();
        let penalties: StdResult<Vec<_>> = PENALTIES
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect();
        
        if let Ok(penalties_list) = penalties {
            for (_, penalty) in penalties_list {
                total_penalties += penalty.amount;
            }
        }
        
        // Get accumulated fees
        let total_fees = get_accumulated_fees(deps.storage)
            .map_err(|e| StdError::generic_err(e.to_string()))?;
        
        let response = StatisticsResponse {
            member_count,
            total_contributions: total_contributions.to_string(),
            total_distributions: total_distributions.to_string(),
            total_penalties: total_penalties.to_string(),
            total_fees: total_fees.to_string(),
            active_rounds,
            completed_rounds,
        };
        
        to_binary(&response)
    }
}
