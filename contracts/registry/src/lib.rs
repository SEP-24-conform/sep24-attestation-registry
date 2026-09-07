#![no_std]

//! On-chain registry of SEP-24 conformance attestations.
//!
//! An off-chain checker (see the `sep24-conformance` and backend repos)
//! independently verifies that a given anchor's SEP-24 implementation
//! conforms to spec, then submits the result here. Wallets and apps can
//! then query this contract directly instead of trusting a centralized
//! list of "verified anchors".
//!
//! This contract deliberately does not re-run any checks itself — it is a
//! tamper-evident record of results computed elsewhere, submitted only by
//! an authorized admin address.

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, String};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Attestation(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Attestation {
    /// Ledger close time (unix seconds) the attestation was written.
    pub timestamp: u64,
    /// Whether the conformance run passed with zero failures.
    pub passed: bool,
    /// Hash of the full conformance report (e.g. sha256 of the JSON report),
    /// so a verifier can confirm a specific report matches this attestation.
    pub result_hash: BytesN<32>,
}

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    /// One-time setup. Panics if already initialized.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized")
    }

    /// Rotates the admin address. Requires the current admin's signature.
    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = Self::get_admin(env.clone());
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Records a conformance attestation for `domain`. Requires the admin's
    /// signature — this is the only way to write to the registry.
    pub fn attest(env: Env, domain: String, passed: bool, result_hash: BytesN<32>) {
        let admin: Address = Self::get_admin(env.clone());
        admin.require_auth();

        let attestation = Attestation {
            timestamp: env.ledger().timestamp(),
            passed,
            result_hash,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Attestation(domain), &attestation);
    }

    /// Looks up the most recent attestation for `domain`, if any.
    pub fn get_attestation(env: Env, domain: String) -> Option<Attestation> {
        env.storage()
            .persistent()
            .get(&DataKey::Attestation(domain))
    }
}

mod test;
