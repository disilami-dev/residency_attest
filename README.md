# residency_attest

## Project Title
residency_attest

## Project Description
`residency_attest` is a Soroban smart contract that turns proof-of-residency
into a portable, privacy-preserving, on-chain credential. A trusted authority
(local government, utility company, university) attests that a wallet holder
lives at a specific physical address by recording a hash of that address along
with an expiry timestamp. KYC services, banks, voting portals and government
e-services can then verify residency in one read call without ever seeing the
underlying address and without having to re-collect documents from the user.

## Project Vision
Today, every new bank account, SIM card, exam registration or subsidy program
forces citizens to upload the same scanned utility bill or rental contract
over and over – each copy a new privacy leak. Our vision is a world where
residency is proven once, by an authoritative issuer, and re-used everywhere
through a single cryptographic check on Stellar.

Long-term, `residency_attest` aims to become the neutral residency-credential
layer of the Stellar identity stack: hash-only on-chain, multi-issuer,
auditable, revocable, and cheap enough (~$0.000003 per verify) that even
micro-services in emerging markets can adopt it.

## Key Features
- **Issuer-signed attestations** – `attest_residency` requires `issuer.require_auth()`, so only authorised authorities can publish residency claims.
- **Privacy by design** – the contract stores only a 32-byte `BytesN<32>` hash (e.g. SHA-256) of the physical address; raw PII never touches the ledger.
- **Expiry & renewal** – every attestation carries a `valid_until` timestamp and can be extended via `renew` without re-issuing a brand-new record.
- **Revocation with reason** – the original issuer can revoke via `revoke(...)` with a short `Symbol` reason that is stored and emitted as an event for audit.
- **One-call KYC gating** – consumers call `verify(resident) -> u32` (0/1/2/3) or the boolean `is_resident(resident)` to instantly check status.
- **Transparent issuer lookup** – `get_issuer`, `expires_at` and `address_hash` let any relying party inspect the credential before trusting it.
- **Event stream** – `ATTEST`, `RENEW` and `REVOKE` events under the `RESID` topic allow off-chain indexers to build dashboards and audit trails.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** identity dApp — see `contracts/residency_attest/src/lib.rs` for the full residency_attest business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CBBHRM2ATAYXLFJGFJ4FJKRRPJGRVRKZ7YMQGN2YDYD66X6DS43LIT2O`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/709890bccd53b9b04f62eab030c0a6cd4046706db5f05b076a04b44a6246acf9`

## Future Scope
- **Multi-issuer trust list** – on-chain registry of approved government / utility issuers, with per-region scopes (country, city, postcode prefix).
- **Tiered residency levels** – distinguish *registered*, *long-term resident*, *citizen* tiers so KYC programs can apply differentiated rules.
- **Zero-knowledge upgrade** – swap the plain `address_hash` for a ZK commitment so users can prove statements like "lives in District 1" without revealing the exact street.
- **Stake-backed issuers** – require issuers to lock XLM that can be slashed on proven false attestations, creating economic accountability.
- **Cross-chain bridge & DID export** – emit W3C Verifiable Credentials so the same residency proof is reusable on EVM chains and traditional DID wallets.
- **Web/mobile companion app** – Freighter-powered portal where residents can view, share and revoke their own attestations and where issuers can batch-attest from a CSV.
- **Mainnet launch** – after testnet hardening and an external security review, promote the contract to Stellar Mainnet with formal governance over the issuer allow-list.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `residency_attest` (identity)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
