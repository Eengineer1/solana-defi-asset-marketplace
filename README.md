# Solana DeFi Asset Marketplace

A production-oriented reference architecture combining an Anchor escrow marketplace, SPL Token/Token-2022 settlement, canonical Metaplex metadata validation, Jupiter-assisted payment routing, a TypeScript SDK, PostgreSQL-backed chain indexer/API, and a minimal operator-facing web surface.

> **Publication note:** This repository contains a generalized and independently publishable implementation derived from work that was previously maintained privately. It has been released publicly following the conclusion of the applicable private agreements. No proprietary client code, confidential information, credentials, or protected implementation details are included.

## What it covers

### On-chain
- Anchor listing PDA and PDA-controlled asset escrow;
- SPL Token and Token-2022 through token interfaces;
- canonical Metaplex Token Metadata PDA/owner validation;
- create, buy and cancel listing lifecycle;
- payment split between seller and marketplace fee recipient;
- state-bound token program IDs, mint relationships and escrow identity;
- explicit events for listing creation, fill and cancellation.

### Off-chain
- deterministic TypeScript PDA/fee SDK;
- Jupiter quote + versioned swap transaction construction;
- confirmed-log subscription and finalized-slot promotion;
- idempotent PostgreSQL event storage;
- Express read API;
- lightweight web/operator surface.

## Repository layout

```text
programs/marketplace/       Anchor program
packages/sdk/               PDAs and deterministic settlement helpers
packages/jupiter-client/    DEX-routing client boundary
packages/indexer/           Solana log ingestion/finality worker
apps/api/                   indexed-data HTTP API
apps/web/                   minimal browser integration surface
db/schema.sql               indexer persistence model
ARCHITECTURE.md              component and trust-boundary design
SECURITY.md                  protocol invariants and hardening
INDEXING.md                  replay/finality strategy
JUPITER.md                   routing integration choices
```

## Why Jupiter stays off-chain

The DEX integration is intentionally compositional rather than embedded into the marketplace program. A buyer can route any supported input asset into the listing's required payment mint through Jupiter, then call the marketplace's deterministic `buy`. This limits the on-chain account surface and keeps route/slippage semantics out of custody logic.

## Metaplex validation

The listing path requires a zero-decimal asset mint and checks that the supplied metadata account is the canonical Metaplex Token Metadata PDA for that mint and is owned by the Metadata program. For collection gating or creator royalties, extend this boundary by decoding and validating the metadata payload rather than trusting client labels.

## Local development

Toolchain target: Anchor `0.32.2`, Agave/Solana `2.3.x`, Node 22+.

```bash
npm install
anchor build
npm test
psql "$DATABASE_URL" -f db/schema.sql
npm run indexer
npm run api
```

Environment variables:

```text
SOLANA_RPC_URL
DATABASE_URL
MARKETPLACE_PROGRAM_ID
PORT
```

## Operational notes

- No mainnet/devnet deployment is claimed until an actual deployment is made and linked.
- No external audit is claimed.
- Indexing distinguishes confirmed observation from finalized state.
- Jupiter responses should be simulated and mint/slippage-checked by the signing client.
- Token-2022 payment mints with transfer fees require an explicit product decision around gross vs net settlement; this implementation treats listing price as gross transfer amount.

## License

MIT.
