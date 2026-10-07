use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_escrow_created(
    e: &Env,
    escrow_id: u64,
    buyer: &Address,
    seller: &Address,
    token: &Address,
    amount: i128,
) {
    e.events().publish(
        (symbol_short!("escrow"), symbol_short!("created")),
        (escrow_id, buyer.clone(), seller.clone(), token.clone(), amount),
    );
}

pub fn emit_escrow_released(e: &Env, escrow_id: u64, seller: &Address, amount: i128) {
    e.events().publish(
        (symbol_short!("escrow"), symbol_short!("release")),
        (escrow_id, seller.clone(), amount),
    );
}

pub fn emit_escrow_refunded(e: &Env, escrow_id: u64, buyer: &Address, amount: i128) {
    e.events().publish(
        (symbol_short!("escrow"), symbol_short!("refund")),
        (escrow_id, buyer.clone(), amount),
    );
}

pub fn emit_escrow_disputed(e: &Env, escrow_id: u64, disputer: &Address) {
    e.events().publish(
        (symbol_short!("escrow"), symbol_short!("dispute")),
        (escrow_id, disputer.clone()),
    );
}

pub fn emit_escrow_resolved(
    e: &Env,
    escrow_id: u64,
    buyer_amount: i128,
    seller_amount: i128,
) {
    e.events().publish(
        (symbol_short!("escrow"), symbol_short!("resolved")),
        (escrow_id, buyer_amount, seller_amount),
    );
}

pub fn emit_spend_cap_set(
    e: &Env,
    owner: &Address,
    delegate: &Address,
    token: &Address,
    per_tx_cap: i128,
    daily_cap: i128,
) {
    e.events().publish(
        (symbol_short!("spendcap"), symbol_short!("set")),
        (owner.clone(), delegate.clone(), token.clone(), per_tx_cap, daily_cap),
    );
}

pub fn emit_delegated_payment(
    e: &Env,
    owner: &Address,
    delegate: &Address,
    recipient: &Address,
    token: &Address,
    amount: i128,
) {
    e.events().publish(
        (symbol_short!("spendcap"), symbol_short!("pay")),
        (owner.clone(), delegate.clone(), recipient.clone(), token.clone(), amount),
    );
}
