use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Pending = 0,
    Released = 1,
    Refunded = 2,
    Disputed = 3,
    Resolved = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub token: Address,
    pub amount: i128,
    pub created_at: u64,
    pub timeout_timestamp: u64,
    pub status: EscrowStatus,
    pub memo: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendCap {
    pub owner: Address,
    pub delegate: Address,
    pub token: Address,
    pub per_tx_cap: i128,
    pub daily_cap: i128,
    pub window_start: u64,
    pub spent_in_window: i128,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    EscrowCounter,
    Escrow(u64),
    SpendCap(Address, Address, Address), // (owner, delegate, token)
    TotalVolume(Address),               // token -> total settled volume
}
