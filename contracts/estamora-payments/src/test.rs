#![cfg(test)]

use crate::{errors::Error, types::EscrowStatus, EstamoraPayments, EstamoraPaymentsClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String,
};

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_id = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &contract_id.address()),
        token::StellarAssetClient::new(e, &contract_id.address()),
    )
}

#[test]
fn test_escrow_lifecycle_create_and_release() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &10_000);

    let contract_id = env.register(EstamoraPayments, ());
    let client = EstamoraPaymentsClient::new(&env, &contract_id);

    client.initialize(&admin);

    assert_eq!(client.get_escrow_count(), 0);

    let amount = 1_000i128;
    let timeout = 3600u64; // 1 hour
    let memo = String::from_str(&env, "Test Milestone Payment");

    let escrow_id = client.create_escrow(
        &buyer,
        &seller,
        &token_client.address,
        &amount,
        &timeout,
        &memo,
    );
    assert_eq!(escrow_id, 1);
    assert_eq!(client.get_escrow_count(), 1);

    // Verify token was transferred from buyer to contract
    assert_eq!(token_client.balance(&buyer), 9_000);
    assert_eq!(token_client.balance(&contract_id), 1_000);

    // Check escrow state
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Pending);
    assert_eq!(escrow.amount, 1_000);
    assert_eq!(escrow.buyer, buyer);
    assert_eq!(escrow.seller, seller);

    // Buyer releases escrow
    client.release_escrow(&buyer, &escrow_id);

    // Check tokens transferred to seller
    assert_eq!(token_client.balance(&seller), 1_000);
    assert_eq!(token_client.balance(&contract_id), 0);

    let updated_escrow = client.get_escrow(&escrow_id);
    assert_eq!(updated_escrow.status, EscrowStatus::Released);
}

#[test]
fn test_escrow_refund_after_timeout() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &5_000);

    let contract_id = env.register(EstamoraPayments, ());
    let client = EstamoraPaymentsClient::new(&env, &contract_id);
    client.initialize(&admin);

    let amount = 2_000i128;
    let timeout = 1_000u64;
    let memo = String::from_str(&env, "Time-locked Order");

    let escrow_id = client.create_escrow(
        &buyer,
        &seller,
        &token_client.address,
        &amount,
        &timeout,
        &memo,
    );

    // Attempting to refund before timeout must fail
    let refund_result = client.try_refund_escrow(&buyer, &escrow_id);
    assert_eq!(refund_result, Err(Ok(Error::TimeoutNotExpired)));

    // Fast-forward ledger timestamp past timeout
    env.ledger().set_timestamp(env.ledger().timestamp() + 1_001);

    // Buyer refunds after timeout
    client.refund_escrow(&buyer, &escrow_id);

    assert_eq!(token_client.balance(&buyer), 5_000);
    assert_eq!(token_client.balance(&contract_id), 0);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Refunded);
}

#[test]
fn test_escrow_dispute_and_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&buyer, &1_000);

    let contract_id = env.register(EstamoraPayments, ());
    let client = EstamoraPaymentsClient::new(&env, &contract_id);
    client.initialize(&admin);

    let escrow_id = client.create_escrow(
        &buyer,
        &seller,
        &token_client.address,
        &1_000,
        &3600,
        &String::from_str(&env, "Freelance Job"),
    );

    // Buyer raises a dispute
    client.dispute_escrow(&buyer, &escrow_id);

    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Disputed);

    // Admin resolves dispute: 60% to seller, 40% to buyer
    client.resolve_dispute(&admin, &escrow_id, &40, &60);

    assert_eq!(token_client.balance(&buyer), 400);
    assert_eq!(token_client.balance(&seller), 600);
    assert_eq!(token_client.balance(&contract_id), 0);

    let final_escrow = client.get_escrow(&escrow_id);
    assert_eq!(final_escrow.status, EscrowStatus::Resolved);
}

#[test]
fn test_spend_cap_delegated_payments() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let delegate = Address::generate(&env); // AI agent or micro-service
    let recipient = Address::generate(&env);

    let (token_client, token_admin) = create_token_contract(&env, &admin);
    token_admin.mint(&owner, &10_000);

    let contract_id = env.register(EstamoraPayments, ());
    let client = EstamoraPaymentsClient::new(&env, &contract_id);
    client.initialize(&admin);

    // Owner approves contract to spend funds up to 5000
    token_client.approve(&owner, &contract_id, &5_000, &10_000);

    // Owner registers spend cap for delegate: max 50 per tx, max 200 per day
    client.register_spend_cap(&owner, &delegate, &token_client.address, &50, &200);

    let cap = client
        .get_spend_cap(&owner, &delegate, &token_client.address)
        .unwrap();
    assert_eq!(cap.per_tx_cap, 50);
    assert_eq!(cap.daily_cap, 200);
    assert!(cap.active);

    // Delegate makes a valid payment of 30
    client.delegated_pay(&delegate, &owner, &recipient, &token_client.address, &30);
    assert_eq!(token_client.balance(&recipient), 30);
    assert_eq!(token_client.balance(&owner), 9_970);

    // Delegate tries to pay 60 (exceeds per_tx_cap of 50) -> should fail
    let tx_fail =
        client.try_delegated_pay(&delegate, &owner, &recipient, &token_client.address, &60);
    assert_eq!(tx_fail, Err(Ok(Error::PerTxCapExceeded)));

    // Delegate makes three more payments of 50 each (total today = 30 + 50 + 50 + 50 = 180)
    client.delegated_pay(&delegate, &owner, &recipient, &token_client.address, &50);
    client.delegated_pay(&delegate, &owner, &recipient, &token_client.address, &50);
    client.delegated_pay(&delegate, &owner, &recipient, &token_client.address, &50);
    assert_eq!(token_client.balance(&recipient), 180);

    // Next payment of 30 would bring total to 210 > 200 daily cap -> should fail
    let daily_fail =
        client.try_delegated_pay(&delegate, &owner, &recipient, &token_client.address, &30);
    assert_eq!(daily_fail, Err(Ok(Error::DailyCapExceeded)));

    // Fast forward 24 hours (86,401 seconds) -> daily window resets
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 86_401);

    // Payment succeeds again in new window
    client.delegated_pay(&delegate, &owner, &recipient, &token_client.address, &40);
    assert_eq!(token_client.balance(&recipient), 220);

    // Revoke spend cap
    client.revoke_spend_cap(&owner, &delegate, &token_client.address);
    let revoked_fail =
        client.try_delegated_pay(&delegate, &owner, &recipient, &token_client.address, &10);
    assert_eq!(revoked_fail, Err(Ok(Error::SpendCapRevoked)));
}
