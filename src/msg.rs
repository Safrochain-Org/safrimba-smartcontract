use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    pub token_denom: String,
    pub contribution_amount: String,
    pub round_frequency: u64, // in seconds
    pub beneficiaries: Vec<String>,
    pub late_penalty: String,
    pub protocol_fees: String,
    pub arbitrator: String,
    pub time_guards: u64, // in seconds
}

#[cw_serde]
pub enum ExecuteMsg {
    // Member management
    RegisterMember { address: String },
    RemoveMember { address: String },
    ReplaceMember { old_address: String, new_address: String },
    
    // Tontine control
    StartTontine {},
    PauseTontine {},
    ResumeTontine {},
    CloseEarly { reason: String },
    
    // Round operations
    DepositContribution {},
    DistributeToBeneficiary {},
    AdvancePayment { beneficiary: String, discount: String },
    
    // Penalty management
    DeclareLate { member: String },
    ApplyPenalty { member: String, amount: String },
    PayPenalty { member: String },
    
    // Fee management
    WithdrawFees {},
    CollectProtocolFees {},
    
    // Dispute resolution
    ResolveDispute { member: String, resolution: String },
    ArbitrateDispute { member: String, decision: String },
    
    // Finalization
    FinalizeTontine {},
    
    // Migration
    Migrate { new_code_id: u64 },
    

}

#[cw_serde]
pub enum QueryMsg {
    // Configuration
    GetConfig {},
    GetAdmin {},
    GetArbitrator {},
    
    // Member information
    GetMembers {},
    GetMember { address: String },
    GetMemberStatus { address: String },
    GetMemberBalance { address: String },
    GetMemberPenalties { address: String },
    
    // Round information
    GetCurrentRound {},
    GetRoundInfo { round: u64 },
    GetRoundDeposits { round: u64 },
    GetRoundState { round: u64 },
    
    // Financial information
    GetTontineBalance {},
    GetRoundBalance { round: u64 },
    GetAccumulatedFees {},
    GetPendingPenalties {},
    
    // Beneficiary information
    GetCurrentBeneficiary {},
    GetNextBeneficiary {},
    GetBeneficiariesList {},
    GetBeneficiarySchedule {},
    
    // Time information
    GetRoundDeadline { round: u64 },
    GetTimeGuards {},
    GetRoundFrequency {},
    
    // Historical information
    GetDistributionHistory {},
    GetPenaltyHistory {},
    GetDepositHistory {},
    
    // State information
    GetTontineState {},
    GetEscrowState {},
    GetDisputeState {},
    
    // Statistics
    GetMemberCount {},
    GetTotalContributions {},
    GetTotalDistributions {},
    GetTotalPenalties {},
    GetTotalFees {},
    GetStatistics {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub token_denom: String,
    pub contribution_amount: String,
    pub round_frequency: u64,
    pub beneficiaries: Vec<String>,
    pub late_penalty: String,
    pub protocol_fees: String,
    pub arbitrator: String,
    pub time_guards: u64,
    pub is_active: bool,
    pub is_paused: bool,
    pub is_finished: bool,
}

#[cw_serde]
pub struct MemberResponse {
    pub address: String,
    pub status: MemberStatus,
    pub balance: String,
    pub penalties: String,
    pub last_contribution: Option<Timestamp>,
    pub is_late: bool,
}

#[cw_serde]
pub struct RoundResponse {
    pub round_number: u64,
    pub state: RoundState,
    pub balance: String,
    pub beneficiary: String,
    pub deadline: Timestamp,
    pub deposits: Vec<DepositResponse>,
    pub is_distributed: bool,
}

#[cw_serde]
pub struct DepositResponse {
    pub member: String,
    pub amount: String,
    pub timestamp: Timestamp,
    pub is_late: bool,
}

#[cw_serde]
pub struct TontineStateResponse {
    pub current_round: u64,
    pub total_rounds: u64,
    pub total_balance: String,
    pub total_fees: String,
    pub total_penalties: String,
    pub member_count: u64,
    pub is_active: bool,
    pub is_paused: bool,
    pub is_finished: bool,
}

#[cw_serde]
pub struct DistributionHistoryResponse {
    pub distributions: Vec<DistributionResponse>,
}

#[cw_serde]
pub struct DistributionResponse {
    pub round: u64,
    pub beneficiary: String,
    pub amount: String,
    pub timestamp: Timestamp,
}

#[cw_serde]
pub struct PenaltyHistoryResponse {
    pub penalties: Vec<PenaltyResponse>,
}

#[cw_serde]
pub struct PenaltyResponse {
    pub member: String,
    pub amount: String,
    pub reason: String,
    pub timestamp: Timestamp,
    pub is_paid: bool,
}

#[cw_serde]
pub struct DepositHistoryResponse {
    pub deposits: Vec<DepositResponse>,
}

#[cw_serde]
pub struct EscrowStateResponse {
    pub is_locked: bool,
    pub locked_amount: String,
    pub lock_reason: String,
    pub lock_timestamp: Option<Timestamp>,
}

#[cw_serde]
pub struct DisputeStateResponse {
    pub active_disputes: Vec<DisputeResponse>,
}

#[cw_serde]
pub struct DisputeResponse {
    pub member: String,
    pub reason: String,
    pub timestamp: Timestamp,
    pub status: DisputeStatus,
    pub resolution: Option<String>,
}

#[cw_serde]
pub enum MemberStatus {
    Active,
    Inactive,
    Suspended,
    Replaced,
    Excluded,
}

#[cw_serde]
pub enum RoundState {
    Pending,
    Active,
    Completed,
    Distributed,
    Failed,
}

#[cw_serde]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
    Closed,
}

#[cw_serde]
pub struct StatisticsResponse {
    pub member_count: u64,
    pub total_contributions: String,
    pub total_distributions: String,
    pub total_penalties: String,
    pub total_fees: String,
    pub active_rounds: u64,
    pub completed_rounds: u64,
}
