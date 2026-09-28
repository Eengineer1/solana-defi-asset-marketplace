# Indexing and reconciliation

The sample indexer separates **observation** from **finality**.

1. `onLogs(..., "confirmed")` gives low-latency signatures.
2. The worker fetches the transaction with `maxSupportedTransactionVersion: 1`.
3. Raw `Program data:` records are persisted idempotently with slot and log index.
4. A periodic finalized-slot check marks rows finalized only once the cluster has finalized their slot.
5. A production deployment should also run a cursor-based backfill/reconciliation loop using signatures for the program and compare reconstructed listing state against `getProgramAccounts`.

Keeping raw event payloads allows event-decoder upgrades without losing the original chain evidence.
