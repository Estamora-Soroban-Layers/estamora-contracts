#![no_std]

pub mod errors;
pub mod events;
pub mod types;

#[cfg(test)]
mod test;

use errors::Error;
use soroban_sdk::{
    contract, contractimpl, token, Address, Env, String,
};
use types::{DataKey, Escrow, EscrowStatus, SpendCap};

const DAY_IN_SECONDS: u64 = 86_400;

#[contract]
pub struct EstamoraPayments;

#[contractimpl]
impl EstamoraPayments {
    /// Initialize the contract with an admin address.
    pub fn initialize(e: Env, admin: Address) -> Result<(), Error> {
        if e.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage().instance().set(&DataKey::EscrowCounter, &0u64);
        Ok(())
    }

    /// Returns the contract admin.
    pub fn get_admin(e: Env) -> Result<Address, Error> {
        e.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Create a milestone / conditional escrow.
    /// Locks tokens from the buyer inside the contract until release or refund.
    pub fn create_escrow(
        e: Env,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: i128,
        timeout_seconds: u64,
        memo: String,
    ) -> Result<u64, Error> {
        buyer.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if timeout_seconds == 0 {
            return Err(Error::InvalidTimeout);
        }

        // Lock funds into the contract
        let token_client = token::Client::new(&e, &token);
        token_client.transfer(&buyer, &e.current_contract_address(), &amount);

        // Increment escrow counter
        let mut counter: u64 = e
            .storage()
            .instance()
            .get(&DataKey::EscrowCounter)
            .unwrap_or(0);
        counter += 1;

        let now = e.ledger().timestamp();
        let timeout_timestamp = now + timeout_seconds;

        let escrow = Escrow {
            id: counter,
            buyer: buyer.clone(),
            seller: seller.clone(),
            token: token.clone(),
            amount,
            created_at: now,
            timeout_timestamp,
            status: EscrowStatus::Pending,
            memo,
        };

        e.storage().persistent().set(&DataKey::Escrow(counter), &escrow);
        e.storage().instance().set(&DataKey::EscrowCounter, &counter);

        events::emit_escrow_created(&e, counter, &buyer, &seller, &token, amount);

        Ok(counter)
    }

    /// Release escrow funds to the seller.
    /// Can be called by the buyer upon milestone completion, or by admin.
    pub fn release_escrow(e: Env, caller: Address, escrow_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut escrow: Escrow = e
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Pending {
            return Err(Error::EscrowNotPending);
        }

        let admin: Address = e
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        // Only buyer or contract admin can approve release
        if caller != escrow.buyer && caller != admin {
            return Err(Error::Unauthorized);
        }

        // Transfer funds from contract to seller
        let token_client = token::Client::new(&e, &escrow.token);
        token_client.transfer(&e.current_contract_address(), &escrow.seller, &escrow.amount);

        escrow.status = EscrowStatus::Released;
        e.storage().persistent().set(&DataKey::Escrow(escrow_id), &escrow);

        events::emit_escrow_released(&e, escrow_id, &escrow.seller, escrow.amount);

        Ok(())
    }

    /// Refund escrow funds back to the buyer.
    /// Can be called by buyer if timeout has expired, or by seller at any time.
    pub fn refund_escrow(e: Env, caller: Address, escrow_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut escrow: Escrow = e
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Pending {
            return Err(Error::EscrowNotPending);
        }

        let admin: Address = e
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        let now = e.ledger().timestamp();

        if caller == escrow.seller || caller == admin {
            // Seller or admin can voluntarily refund at any time
        } else if caller == escrow.buyer {
            // Buyer can only refund if timeout has passed
            if now < escrow.timeout_timestamp {
                return Err(Error::TimeoutNotExpired);
            }
        } else {
            return Err(Error::Unauthorized);
        }

        // Transfer funds from contract back to buyer
        let token_client = token::Client::new(&e, &escrow.token);
        token_client.transfer(&e.current_contract_address(), &escrow.buyer, &escrow.amount);

        escrow.status = EscrowStatus::Refunded;
        e.storage().persistent().set(&DataKey::Escrow(escrow_id), &escrow);

        events::emit_escrow_refunded(&e, escrow_id, &escrow.buyer, escrow.amount);

        Ok(())
    }

    /// Flag an escrow as disputed by either buyer or seller.
    pub fn dispute_escrow(e: Env, caller: Address, escrow_id: u64) -> Result<(), Error> {
        caller.require_auth();

        let mut escrow: Escrow = e
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Pending {
            return Err(Error::EscrowNotPending);
        }

        if caller != escrow.buyer && caller != escrow.seller {
            return Err(Error::Unauthorized);
        }

        escrow.status = EscrowStatus::Disputed;
        e.storage().persistent().set(&DataKey::Escrow(escrow_id), &escrow);

        events::emit_escrow_disputed(&e, escrow_id, &caller);

        Ok(())
    }

    /// Resolve a dispute by splitting funds between buyer and seller according to percentage.
    pub fn resolve_dispute(
        e: Env,
        admin: Address,
        escrow_id: u64,
        buyer_pct: u32,
        seller_pct: u32,
    ) -> Result<(), Error> {
        admin.require_auth();

        let contract_admin: Address = e
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if admin != contract_admin {
            return Err(Error::Unauthorized);
        }

        if buyer_pct + seller_pct != 100 {
            return Err(Error::InvalidSplitPercentage);
        }

        let mut escrow: Escrow = e
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::EscrowNotFound)?;

        if escrow.status != EscrowStatus::Disputed {
            return Err(Error::EscrowNotDisputed);
        }

        let buyer_amount = (escrow.amount * (buyer_pct as i128)) / 100;
        let seller_amount = escrow.amount - buyer_amount;

        let token_client = token::Client::new(&e, &escrow.token);

        if buyer_amount > 0 {
            token_client.transfer(&e.current_contract_address(), &escrow.buyer, &buyer_amount);
        }
        if seller_amount > 0 {
            token_client.transfer(&e.current_contract_address(), &escrow.seller, &seller_amount);
        }

        escrow.status = EscrowStatus::Resolved;
        e.storage().persistent().set(&DataKey::Escrow(escrow_id), &escrow);

        events::emit_escrow_resolved(&e, escrow_id, buyer_amount, seller_amount);

        Ok(())
    }

    /// Register a delegated spend cap for an agent or secondary account.
    pub fn register_spend_cap(
        e: Env,
        owner: Address,
        delegate: Address,
        token: Address,
        per_tx_cap: i128,
        daily_cap: i128,
    ) -> Result<(), Error> {
        owner.require_auth();

        if per_tx_cap <= 0 || daily_cap < per_tx_cap {
            return Err(Error::InvalidAmount);
        }

        let now = e.ledger().timestamp();
        let key = DataKey::SpendCap(owner.clone(), delegate.clone(), token.clone());

        let cap = SpendCap {
            owner: owner.clone(),
            delegate: delegate.clone(),
            token: token.clone(),
            per_tx_cap,
            daily_cap,
            window_start: now,
            spent_in_window: 0,
            active: true,
        };

        e.storage().persistent().set(&key, &cap);

        events::emit_spend_cap_set(&e, &owner, &delegate, &token, per_tx_cap, daily_cap);

        Ok(())
    }

    /// Revoke a delegated spend cap.
    pub fn revoke_spend_cap(
        e: Env,
        owner: Address,
        delegate: Address,
        token: Address,
    ) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::SpendCap(owner.clone(), delegate.clone(), token.clone());
        let mut cap: SpendCap = e
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::SpendCapNotFound)?;

        cap.active = false;
        e.storage().persistent().set(&key, &cap);

        Ok(())
    }

    /// Execute a delegated payment under an active spend cap.
    /// Transfers approved allowance tokens from owner to recipient.
    pub fn delegated_pay(
        e: Env,
        delegate: Address,
        owner: Address,
        recipient: Address,
        token: Address,
        amount: i128,
    ) -> Result<(), Error> {
        delegate.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let key = DataKey::SpendCap(owner.clone(), delegate.clone(), token.clone());
        let mut cap: SpendCap = e
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::SpendCapNotFound)?;

        if !cap.active {
            return Err(Error::SpendCapRevoked);
        }

        if amount > cap.per_tx_cap {
            return Err(Error::PerTxCapExceeded);
        }

        let now = e.ledger().timestamp();

        // Check 24-hour rolling window
        if now >= cap.window_start + DAY_IN_SECONDS {
            cap.window_start = now;
            cap.spent_in_window = 0;
        }

        if cap.spent_in_window + amount > cap.daily_cap {
            return Err(Error::DailyCapExceeded);
        }

        cap.spent_in_window += amount;
        e.storage().persistent().set(&key, &cap);

        // Execute token transfer using allowance
        let token_client = token::Client::new(&e, &token);
        token_client.transfer_from(&e.current_contract_address(), &owner, &recipient, &amount);

        events::emit_delegated_payment(&e, &owner, &delegate, &recipient, &token, amount);

        Ok(())
    }

    /// Query an escrow by ID.
    pub fn get_escrow(e: Env, escrow_id: u64) -> Result<Escrow, Error> {
        e.storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .ok_or(Error::EscrowNotFound)
    }

    /// Query spend cap for a delegate.
    pub fn get_spend_cap(
        e: Env,
        owner: Address,
        delegate: Address,
        token: Address,
    ) -> Option<SpendCap> {
        e.storage()
            .persistent()
            .get(&DataKey::SpendCap(owner, delegate, token))
    }

    /// Query total escrows created.
    pub fn get_escrow_count(e: Env) -> u64 {
        e.storage()
            .instance()
            .get(&DataKey::EscrowCounter)
            .unwrap_or(0)
    }
}
