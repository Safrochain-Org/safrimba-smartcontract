use cosmwasm_std::{
    Deps, DepsMut, Env, MessageInfo, Response, StdResult, Binary
};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, initialize_state,
    validate_config, validate_amount, CONFIG
};
use crate::execute::ExecuteHandler;

pub struct Contract;

impl Contract {
    pub fn new() -> Self {
        Contract
    }

    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        // Validate configuration
        let config = Config {
            admin: deps.api.addr_validate(&msg.admin)?,
            token_denom: msg.token_denom,
            contribution_amount: validate_amount(&msg.contribution_amount)?,
            round_frequency: msg.round_frequency,
            beneficiaries: msg.beneficiaries.iter()
                .map(|addr| deps.api.addr_validate(addr))
                .collect::<Result<Vec<_>, _>>()?,
            late_penalty: validate_amount(&msg.late_penalty)?,
            protocol_fees: validate_amount(&msg.protocol_fees)?,
            arbitrator: deps.api.addr_validate(&msg.arbitrator)?,
            time_guards: msg.time_guards,
        };

        validate_config(&config)?;

        // Save configuration
        CONFIG.save(deps.storage, &config)?;

        // Initialize state
        initialize_state(deps.storage)?;

        // Create response with events
        let response = Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("admin", config.admin.to_string())
            .add_attribute("token_denom", config.token_denom)
            .add_attribute("contribution_amount", config.contribution_amount.to_string())
            .add_attribute("round_frequency", config.round_frequency.to_string())
            .add_attribute("beneficiaries_count", config.beneficiaries.len().to_string())
            .add_attribute("late_penalty", config.late_penalty.to_string())
            .add_attribute("protocol_fees", config.protocol_fees.to_string())
            .add_attribute("arbitrator", config.arbitrator.to_string())
            .add_attribute("time_guards", config.time_guards.to_string());

        Ok(response)
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        let result = match msg {
            ExecuteMsg::RegisterMember { address } => {
                ExecuteHandler::register_member(deps, env, info, address)
            }
            ExecuteMsg::RemoveMember { address } => {
                ExecuteHandler::remove_member(deps, env, info, address)
            }
            ExecuteMsg::ReplaceMember { old_address, new_address } => {
                ExecuteHandler::replace_member(deps, env, info, old_address, new_address)
            }
            ExecuteMsg::StartTontine {} => {
                ExecuteHandler::start_tontine(deps, env, info)
            }
            ExecuteMsg::PauseTontine {} => {
                ExecuteHandler::pause_tontine(deps, env, info)
            }
            ExecuteMsg::ResumeTontine {} => {
                ExecuteHandler::resume_tontine(deps, env, info)
            }
            ExecuteMsg::CloseEarly { reason } => {
                ExecuteHandler::close_early(deps, env, info, reason)
            }
            ExecuteMsg::DepositContribution {} => {
                ExecuteHandler::deposit_contribution(deps, env, info)
            }
            ExecuteMsg::DistributeToBeneficiary {} => {
                ExecuteHandler::distribute_to_beneficiary(deps, env, info)
            }
            ExecuteMsg::AdvancePayment { beneficiary, discount } => {
                ExecuteHandler::advance_payment(deps, env, info, beneficiary, discount)
            }
            ExecuteMsg::DeclareLate { member } => {
                ExecuteHandler::declare_late(deps, env, info, member)
            }
            ExecuteMsg::ApplyPenalty { member, amount } => {
                ExecuteHandler::apply_penalty(deps, env, info, member, amount)
            }
            ExecuteMsg::PayPenalty { member } => {
                ExecuteHandler::pay_penalty(deps, env, info, member)
            }
            ExecuteMsg::WithdrawFees {} => {
                ExecuteHandler::withdraw_fees(deps, env, info)
            }
            ExecuteMsg::CollectProtocolFees {} => {
                ExecuteHandler::collect_protocol_fees(deps, env, info)
            }
            ExecuteMsg::ResolveDispute { member, resolution } => {
                ExecuteHandler::resolve_dispute(deps, env, info, member, resolution)
            }
            ExecuteMsg::ArbitrateDispute { member, decision } => {
                ExecuteHandler::arbitrate_dispute(deps, env, info, member, decision)
            }
            ExecuteMsg::FinalizeTontine {} => {
                ExecuteHandler::finalize_tontine(deps, env, info)
            }
            ExecuteMsg::Migrate { new_code_id } => {
                ExecuteHandler::migrate(deps, env, info, new_code_id)
            }
        };

        result
    }

    pub fn query(&self, deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        crate::query::QueryHandler::handle_query(deps, msg)
    }
}
