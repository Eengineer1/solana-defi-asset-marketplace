import { Connection, PublicKey } from "@solana/web3.js";
import pg from "pg";

const { Pool } = pg;
const rpc = process.env.SOLANA_RPC_URL ?? "http://127.0.0.1:8899";
const databaseUrl = process.env.DATABASE_URL ?? "postgres://postgres:postgres@127.0.0.1:5432/marketplace";
const programId = new PublicKey(process.env.MARKETPLACE_PROGRAM_ID ?? "BDGYk9AwrWG8wKey93VzpetV8d2qhMoEbPyi89FJ9eCP");
const connection = new Connection(rpc, "confirmed");
const db = new Pool({ connectionString: databaseUrl });

async function ingestSignature(signature: string, slot: number) {
  const tx = await connection.getTransaction(signature, {
    commitment: "confirmed",
    maxSupportedTransactionVersion: 1,
  });
  if (!tx) return;
  const logs = tx.meta?.logMessages ?? [];
  let logIndex = 0;
  for (const line of logs) {
    if (!line.startsWith("Program data:")) continue;
    await db.query(
      `insert into market_events(signature, slot, log_index, program_id, event_kind, payload, block_time)
       values ($1,$2,$3,$4,$5,$6,$7)
       on conflict (signature, log_index) do nothing`,
      [signature, slot, logIndex++, programId.toBase58(), "anchor_event_raw", JSON.stringify({ line }), tx.blockTime],
    );
  }
  await db.query(
    `insert into chain_cursor(name, slot, signature) values ('confirmed', $1, $2)
     on conflict(name) do update set slot=excluded.slot, signature=excluded.signature, updated_at=now()`,
    [slot, signature],
  );
}

async function markFinalized() {
  const finalized = await connection.getSlot("finalized");
  await db.query(`update market_events set finalized=true where slot <= $1 and finalized=false`, [finalized]);
}

connection.onLogs(programId, (entry, ctx) => {
  void ingestSignature(entry.signature, ctx.slot).catch((error) => console.error("ingest failed", error));
}, "confirmed");

setInterval(() => void markFinalized().catch((error) => console.error("finalization failed", error)), 15_000);
console.log(`indexing ${programId.toBase58()} from ${rpc}`);
