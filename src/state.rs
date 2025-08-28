use cosmwasm_std::{
    Addr, Storage, Timestamp, Uint128
};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use crate::error::ContractError;
use crate::msg::{MemberStatus, RoundState, DisputeStatus};

// Configuration - immutable after instantiation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub admin: Addr,
    pub token_denom: String,
    pub contribution_amount: Uint128,
    pub round_frequency: u64, // in seconds
    pub beneficiaries: Vec<Addr>,
    pub late_penalty: Uint128,
    pub protocol_fees: Uint128,
    pub arbitrator: Addr,
    pub time_guards: u64, // in seconds
}

// Tontine state - mutable
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TontineState {
    pub is_active: bool,
    pub is_paused: bool,
    pub is_finished: bool,
    pub current_round: u64,
    pub total_rounds: u64,
    pub start_time: Option<Timestamp>,
    pub last_round_time: Option<Timestamp>,
}

// Member information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Member {
    pub address: Addr,
    pub status: MemberStatus,
    pub balance: Uint128,
    pub penalties: Uint128,
    pub last_contribution: Option<Timestamp>,
    pub is_late: bool,
    pub registration_time: Timestamp,
}

// Round information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Round {
    pub round_number: u64,
    pub state: RoundState,
    pub balance: Uint128,
    pub beneficiary: Addr,
    pub deadline: Timestamp,
    pub deposits: Vec<Deposit>,
    pub is_distributed: bool,
    pub distribution_time: Option<Timestamp>,
}

// Deposit information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Deposit {
    pub member: Addr,
    pub amount: Uint128,
    pub timestamp: Timestamp,
    pub is_late: bool,
}

// Penalty information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Penalty {
    pub member: Addr,
    pub amount: Uint128,
    pub reason: String,
    pub timestamp: Timestamp,
    pub is_paid: bool,
    pub payment_time: Option<Timestamp>,
}

// Distribution information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Distribution {
    pub round: u64,
    pub beneficiary: Addr,
    pub amount: Uint128,
    pub timestamp: Timestamp,
}

// Dispute information
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Dispute {
    pub member: Addr,
    pub reason: String,
    pub timestamp: Timestamp,
    pub status: DisputeStatus,
    pub resolution: Option<String>,
    pub resolution_time: Option<Timestamp>,
}

// Escrow state
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EscrowState {
    pub is_locked: bool,
    pub locked_amount: Uint128,
    pub lock_reason: String,
    pub lock_timestamp: Option<Timestamp>,
}

// Storage keys
pub const CONFIG: Item<Config> = Item::new("config");
pub const TONTINE_STATE: Item<TontineState> = Item::new("tontine_state");
pub const ESCROW_STATE: Item<EscrowState> = Item::new("escrow_state");

// Member storage with indexes


pub fn members<'a>() -> Map<'a, &'a str, Member> {
    Map::new("members")
}

// Round storage
pub const ROUNDS: Map<u64, Round> = Map::new("rounds");

// Penalty storage
pub const PENALTIES: Map<(&Addr, u64), Penalty> = Map::new("penalties");

// Distribution storage
pub const DISTRIBUTIONS: Map<u64, Distribution> = Map::new("distributions");

// Dispute storage
pub const DISPUTES: Map<(&Addr, u64), Dispute> = Map::new("disputes");

// Fee storage
pub const ACCUMULATED_FEES: Item<Uint128> = Item::new("accumulated_fees");

// State management functions
pub fn initialize_state(storage: &mut dyn Storage) -> Result<(), ContractError> {
    let tontine_state = TontineState {
        is_active: false,
        is_paused: false,
        is_finished: false,
        current_round: 0,
        total_rounds: 0,
        start_time: None,
        last_round_time: None,
    };
    TONTINE_STATE.save(storage, &tontine_state)?;

    let escrow_state = EscrowState {
        is_locked: false,
        locked_amount: Uint128::zero(),
        lock_reason: String::new(),
        lock_timestamp: None,
    };
    ESCROW_STATE.save(storage, &escrow_state)?;

    ACCUMULATED_FEES.save(storage, &Uint128::zero())?;

    Ok(())
}

pub fn get_config(storage: &dyn Storage) -> Result<Config, ContractError> {
    CONFIG.load(storage).map_err(|_| ContractError::InvalidConfiguration { 
        msg: "Configuration not found".to_string() 
    })
}

pub fn get_tontine_state(storage: &dyn Storage) -> Result<TontineState, ContractError> {
    TONTINE_STATE.load(storage).map_err(|_| ContractError::InvalidStateUpdate { 
        msg: "Tontine state not found".to_string() 
    })
}

pub fn get_escrow_state(storage: &dyn Storage) -> Result<EscrowState, ContractError> {
    ESCROW_STATE.load(storage).map_err(|_| ContractError::InvalidEscrowState)
}

pub fn get_member(storage: &dyn Storage, address: &Addr) -> Result<Member, ContractError> {
    members().load(storage, address.as_str()).map_err(|_| ContractError::MemberNotFound { 
        address: address.to_string() 
    })
}

pub fn get_round(storage: &dyn Storage, round_number: u64) -> Result<Round, ContractError> {
    ROUNDS.load(storage, round_number).map_err(|_| ContractError::InvalidRound { round: round_number })
}

pub fn get_current_round(storage: &dyn Storage) -> Result<Round, ContractError> {
    let state = get_tontine_state(storage)?;
    if state.current_round == 0 {
        return Err(ContractError::NoActiveRound);
    }
    get_round(storage, state.current_round)
}

pub fn get_accumulated_fees(storage: &dyn Storage) -> Result<Uint128, ContractError> {
    ACCUMULATED_FEES.load(storage).map_err(|_| ContractError::InvalidFeeManagement { 
        msg: "Accumulated fees not found".to_string() 
    })
}

// Validation functions
pub fn validate_config(config: &Config) -> Result<(), ContractError> {
    if config.contribution_amount == Uint128::zero() {
        return Err(ContractError::InvalidContributionAmount);
    }
    if config.round_frequency == 0 {
        return Err(ContractError::InvalidRoundFrequency);
    }
    if config.beneficiaries.is_empty() {
        return Err(ContractError::InvalidBeneficiariesList);
    }
    if config.late_penalty >= config.contribution_amount {
        return Err(ContractError::InvalidLatePenaltyAmount);
    }
    if config.protocol_fees >= config.contribution_amount {
        return Err(ContractError::InvalidProtocolFeesAmount);
    }
    if config.time_guards == 0 {
        return Err(ContractError::InvalidTimeGuards);
    }
    Ok(())
}

pub fn validate_member_address(address: &str) -> Result<(), ContractError> {
    if address.is_empty() {
        return Err(ContractError::InvalidMemberManagement { 
            msg: "Member address cannot be empty".to_string() 
        });
    }
    
    // Validate bech32 format for Cosmos addresses
    if !address.starts_with("addr_safro") {
        return Err(ContractError::InvalidMemberManagement { 
            msg: "Member address must be a valid Safrochain address (addr_safro...)".to_string() 
        });
    }
    
    // Basic length validation for bech32 addresses
    if address.len() < 20 || address.len() > 50 {
        return Err(ContractError::InvalidMemberManagement { 
            msg: "Member address length is invalid".to_string() 
        });
    }
    
    Ok(())
}

pub fn validate_amount(amount: &str) -> Result<Uint128, ContractError> {
    amount.parse::<Uint128>().map_err(|_| ContractError::InvalidAmount { 
        msg: "Invalid amount format".to_string() 
    })
}
