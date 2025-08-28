use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {msg}")]
    Unauthorized { msg: String },

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },

    #[error("Invalid amount: {msg}")]
    InvalidAmount { msg: String },

    #[error("Member not found: {address}")]
    MemberNotFound { address: String },

    #[error("Member already exists: {address}")]
    MemberAlreadyExists { address: String },

    #[error("Tontine not started")]
    TontineNotStarted,

    #[error("Tontine already started")]
    TontineAlreadyStarted,

    #[error("Tontine already finished")]
    TontineAlreadyFinished,

    #[error("Round not active")]
    RoundNotActive,

    #[error("Round deadline not reached")]
    RoundDeadlineNotReached,

    #[error("Round deadline exceeded")]
    RoundDeadlineExceeded,

    #[error("No active round")]
    NoActiveRound,

    #[error("Invalid beneficiary index: {index}")]
    InvalidBeneficiaryIndex { index: u64 },

    #[error("Invalid round: {round}")]
    InvalidRound { round: u64 },

    #[error("Invalid token denomination: {denom}")]
    InvalidTokenDenom { denom: String },

    #[error("Invalid contribution amount")]
    InvalidContributionAmount,

    #[error("Invalid late penalty amount")]
    InvalidLatePenaltyAmount,

    #[error("Invalid protocol fees amount")]
    InvalidProtocolFeesAmount,

    #[error("Invalid time guards")]
    InvalidTimeGuards,

    #[error("Invalid round frequency")]
    InvalidRoundFrequency,

    #[error("Invalid beneficiaries list")]
    InvalidBeneficiariesList,

    #[error("Member already contributed to this round")]
    MemberAlreadyContributed,

    #[error("Member has pending penalties")]
    MemberHasPenalties,

    #[error("Member is late")]
    MemberIsLate,

    #[error("Cannot replace member during active round")]
    CannotReplaceDuringActiveRound,

    #[error("Cannot advance payment: {msg}")]
    CannotAdvancePayment { msg: String },

    #[error("Cannot close early: {msg}")]
    CannotCloseEarly { msg: String },

    #[error("Invalid migration: {msg}")]
    InvalidMigration { msg: String },

    #[error("Invalid escrow state")]
    InvalidEscrowState,

    #[error("Dispute resolution failed: {msg}")]
    DisputeResolutionFailed { msg: String },

    #[error("Invalid round state: {state}")]
    InvalidRoundState { state: String },

    #[error("Invalid member state: {state}")]
    InvalidMemberState { state: String },

    #[error("Invalid deposit: {msg}")]
    InvalidDeposit { msg: String },

    #[error("Invalid withdrawal: {msg}")]
    InvalidWithdrawal { msg: String },

    #[error("Invalid distribution: {msg}")]
    InvalidDistribution { msg: String },

    #[error("Invalid penalty application: {msg}")]
    InvalidPenaltyApplication { msg: String },

    #[error("Invalid fee collection: {msg}")]
    InvalidFeeCollection { msg: String },

    #[error("Invalid member replacement: {msg}")]
    InvalidMemberReplacement { msg: String },

    #[error("Invalid early closure: {msg}")]
    InvalidEarlyClosure { msg: String },

    #[error("Invalid finalization: {msg}")]
    InvalidFinalization { msg: String },

    #[error("Invalid CW20 receive: {msg}")]
    InvalidCw20Receive { msg: String },

    #[error("Invalid native funds: {msg}")]
    InvalidNativeFunds { msg: String },

    #[error("Invalid event: {msg}")]
    InvalidEvent { msg: String },

    #[error("Invalid state update: {msg}")]
    InvalidStateUpdate { msg: String },

    #[error("Invalid configuration: {msg}")]
    InvalidConfiguration { msg: String },

    #[error("Invalid member management: {msg}")]
    InvalidMemberManagement { msg: String },

    #[error("Invalid round management: {msg}")]
    InvalidRoundManagement { msg: String },

    #[error("Invalid penalty management: {msg}")]
    InvalidPenaltyManagement { msg: String },

    #[error("Invalid fee management: {msg}")]
    InvalidFeeManagement { msg: String },

    #[error("Invalid arbitration: {msg}")]
    InvalidArbitration { msg: String },

    #[error("Invalid time management: {msg}")]
    InvalidTimeManagement { msg: String },

    #[error("Invalid escrow management: {msg}")]
    InvalidEscrowManagement { msg: String },

    #[error("Invalid dispute resolution: {msg}")]
    InvalidDisputeResolution { msg: String },

}
