# Security model

## Invariants

- one active listing PDA exists per asset mint;
- the escrow token account is owned by the listing PDA and bound to the listed asset mint;
- the seller on a buy/cancel path is the seller stored in listing state;
- source/destination payment accounts share the configured payment mint;
- token-program IDs are pinned in listing state and compared on settlement;
- the metadata account is the canonical Metaplex metadata PDA owned by the Token Metadata program;
- listing state closes only after the asset transfer succeeds, because Solana transactions are atomic;
- marketplace fee basis points are capped at initialization.

## Jupiter trust boundary

Jupiter quote/swap construction is a client concern, not an on-chain authority. The client should simulate returned transactions, apply user-configured slippage, verify input/output mints and display route/price-impact data before signing.

## Production requirements

This repository is not represented as externally audited or mainnet deployed. Production rollout should add independent review, full local-validator E2E coverage, token-extension compatibility tests, upgrade-authority policy, RPC redundancy, indexer replay/backfill, rate limiting and wallet transaction UX hardening.
