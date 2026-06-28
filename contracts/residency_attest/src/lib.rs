#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol,
};

/// On-chain record of a residency attestation issued by a trusted authority
/// (e.g. local government, utility company) for a given resident wallet.
///
/// `address_hash` is a 32-byte digest (e.g. SHA-256) of the resident's
/// physical address so the contract never stores raw PII on-chain.
#[contracttype]
#[derive(Clone)]
pub struct Attestation {
    pub issuer: Address,
    pub address_hash: BytesN<32>,
    pub issued_at: u64,
    pub valid_until: u64,
    pub revoked: bool,
    pub revoke_reason: Symbol,
}

/// Persistent storage keys.
#[contracttype]
pub enum DataKey {
    /// Attestation record keyed by resident wallet address.
    Attest(Address),
}

/// Event topic prefix used for all residency-attestation events.
const EVT: Symbol = symbol_short!("RESID");

#[contract]
pub struct ResidencyAttest;

#[contractimpl]
impl ResidencyAttest {
    /// Issue a fresh residency attestation for `resident`.
    ///
    /// * `issuer`        – authorised attesting party (must sign the tx).
    /// * `resident`      – wallet identifying the person living at the address.
    /// * `address_hash`  – 32-byte hash of the physical address (privacy-preserving).
    /// * `valid_until`   – unix timestamp (seconds) when the attestation expires.
    ///
    /// Panics if an attestation already exists for `resident` – use `renew`
    /// or `revoke` instead. Emits a `("RESID","ATTEST")` event.
    pub fn attest_residency(
        env: Env,
        issuer: Address,
        resident: Address,
        address_hash: BytesN<32>,
        valid_until: u64,
    ) {
        issuer.require_auth();

        let now = env.ledger().timestamp();
        if valid_until <= now {
            panic!("valid_until must be in the future");
        }

        let key = DataKey::Attest(resident.clone());
        if env.storage().persistent().has(&key) {
            panic!("attestation already exists; renew or revoke first");
        }

        let attest = Attestation {
            issuer: issuer.clone(),
            address_hash,
            issued_at: now,
            valid_until,
            revoked: false,
            revoke_reason: symbol_short!("NONE"),
        };

        env.storage().persistent().set(&key, &attest);
        env.events().publish(
            (EVT, symbol_short!("ATTEST")),
            (issuer, resident, valid_until),
        );
    }

    /// Revoke an existing attestation (e.g. resident moved away or fraud detected).
    ///
    /// Only the original issuer may revoke. `reason` is a short symbol that is
    /// stored on-chain and emitted in the `("RESID","REVOKE")` event so KYC
    /// services can audit why a previously-valid attestation is no longer trusted.
    pub fn revoke(env: Env, issuer: Address, resident: Address, reason: Symbol) {
        issuer.require_auth();

        let key = DataKey::Attest(resident.clone());
        let mut attest: Attestation = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no attestation for this resident");

        if attest.issuer != issuer {
            panic!("only the original issuer may revoke");
        }
        if attest.revoked {
            panic!("attestation already revoked");
        }

        attest.revoked = true;
        attest.revoke_reason = reason.clone();
        env.storage().persistent().set(&key, &attest);

        env.events().publish(
            (EVT, symbol_short!("REVOKE")),
            (issuer, resident, reason),
        );
    }

    /// Renew an attestation with a new expiry. Only the original issuer may renew.
    /// Renewing also clears any prior `revoked` flag and resets the reason – useful
    /// when a resident temporarily lost status and then re-qualified.
    pub fn renew(env: Env, issuer: Address, resident: Address, new_valid_until: u64) {
        issuer.require_auth();

        let now = env.ledger().timestamp();
        if new_valid_until <= now {
            panic!("new_valid_until must be in the future");
        }

        let key = DataKey::Attest(resident.clone());
        let mut attest: Attestation = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no attestation for this resident");

        if attest.issuer != issuer {
            panic!("only the original issuer may renew");
        }

        attest.valid_until = new_valid_until;
        attest.revoked = false;
        attest.revoke_reason = symbol_short!("NONE");
        env.storage().persistent().set(&key, &attest);

        env.events().publish(
            (EVT, symbol_short!("RENEW")),
            (issuer, resident, new_valid_until),
        );
    }

    /// Verify the status of a resident's attestation.
    ///
    /// Returns:
    /// * `0` – no attestation has ever been issued for this address.
    /// * `1` – valid: not revoked and not expired.
    /// * `2` – expired: `valid_until` is in the past.
    /// * `3` – revoked by the issuer.
    pub fn verify(env: Env, resident: Address) -> u32 {
        let key = DataKey::Attest(resident);
        match env
            .storage()
            .persistent()
            .get::<DataKey, Attestation>(&key)
        {
            None => 0,
            Some(a) => {
                if a.revoked {
                    3
                } else if env.ledger().timestamp() > a.valid_until {
                    2
                } else {
                    1
                }
            }
        }
    }

    /// Convenience boolean – true iff the resident has a currently valid,
    /// non-expired, non-revoked attestation. Intended for fast KYC gating.
    pub fn is_resident(env: Env, resident: Address) -> bool {
        Self::verify(env, resident) == 1
    }

    /// Return the issuing authority's address for `resident`.
    /// Panics if no attestation exists.
    pub fn get_issuer(env: Env, resident: Address) -> Address {
        let key = DataKey::Attest(resident);
        let attest: Attestation = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no attestation for this resident");
        attest.issuer
    }

    /// Return the unix expiry timestamp (seconds) of a resident's attestation.
    /// Panics if no attestation exists.
    pub fn expires_at(env: Env, resident: Address) -> u64 {
        let key = DataKey::Attest(resident);
        let attest: Attestation = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no attestation for this resident");
        attest.valid_until
    }

    /// Return the privacy-preserving address hash registered for `resident`.
    /// External KYC services can hash a claimed physical address and compare
    /// to this value to confirm the resident lives where they claim to.
    /// Panics if no attestation exists.
    pub fn address_hash(env: Env, resident: Address) -> BytesN<32> {
        let key = DataKey::Attest(resident);
        let attest: Attestation = env
            .storage()
            .persistent()
            .get(&key)
            .expect("no attestation for this resident");
        attest.address_hash
    }
}
