# Architecture

## Components

```text
wallet / web client
      |
      +--> TypeScript SDK ----------> Anchor marketplace program
      |                                   |       |
      |                                   |       +--> SPL Token / Token-2022
      |                                   +----------> canonical Metaplex metadata PDA check
      |
      +--> Jupiter client ----------> quote + swap transaction construction
      |
      +--> API <-------------------- PostgreSQL <---- finalized/confirmed event indexer
```

## On-chain boundaries

The marketplace program owns no user wallet keys. Each active listing is a PDA derived from the NFT mint and controls a dedicated associated-token escrow account. The listing stores seller, asset/payment mints, escrow address and the two token-program IDs selected at creation. Buy/cancel repeat those relationships as account constraints.

The asset mint must have zero decimals and the caller must provide the canonical Metaplex metadata PDA. The program validates both the PDA derivation and Metadata-program ownership. It does not rely on a client-supplied “is NFT” flag.

## Settlement

Payment is gross-denominated in the listing's payment mint. The marketplace fee is bounded to 10% at market initialization and split deterministically in basis points. Settlement uses token-interface CPIs, so the configured payment mint can belong to SPL Token or Token-2022. If the chosen mint itself applies transfer fees, the economic agreement should explicitly define whether quoted price is gross or net; this implementation treats it as gross transfer amount.

## Jupiter integration

Jupiter is deliberately kept off-chain. `packages/jupiter-client` obtains a quote and builds a versioned swap transaction so a user can acquire the required settlement mint before calling `buy`. The Anchor program does not accept arbitrary Jupiter instructions or remaining accounts, reducing CPI attack surface and keeping marketplace settlement deterministic.

## Indexing

The indexer subscribes to program logs at `confirmed`, fetches versioned transactions, stores raw Anchor event payloads idempotently by `(signature, log_index)`, and separately advances finalization. Raw events are retained so decoding logic can be upgraded without losing source evidence.
