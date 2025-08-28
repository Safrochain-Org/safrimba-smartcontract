pub mod contract;
pub mod error;
pub mod msg;
pub mod state;
pub mod execute;
pub mod query;


use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::contract::Contract;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

const CONTRACT_NAME: &str = "crates.io:tontine-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let contract = Contract::new();
    contract.instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let contract = Contract::new();
    contract.execute(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let contract = Contract::new();
    contract.query(deps, msg)
}

// CW20 tokens are handled through the execute entry point

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "usaf"));
        
        let msg = InstantiateMsg {
            admin: "admin".to_string(),
            token_denom: "usaf".to_string(),
            contribution_amount: "1000".to_string(),
            round_frequency: 86400, // 1 day in seconds
            beneficiaries: vec!["beneficiary1".to_string(), "beneficiary2".to_string()],
            late_penalty: "50".to_string(),
            protocol_fees: "10".to_string(),
            arbitrator: "arbitrator".to_string(),
            time_guards: 3600, // 1 hour in seconds
        };

        let result = instantiate(deps.as_mut(), env, info, msg);
        assert!(result.is_ok());
    }
}
