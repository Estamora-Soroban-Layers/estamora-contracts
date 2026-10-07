use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidTimeout = 5,
    EscrowNotFound = 6,
    EscrowNotPending = 7,
    TimeoutNotExpired = 8,
    SpendCapNotFound = 9,
    PerTxCapExceeded = 10,
    DailyCapExceeded = 11,
    SpendCapRevoked = 12,
    InvalidSplitPercentage = 13,
    EscrowNotDisputed = 14,
}
