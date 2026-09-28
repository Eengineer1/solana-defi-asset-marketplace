import express from "express";
import pg from "pg";

const { Pool } = pg;
const app = express();
const db = new Pool({ connectionString: process.env.DATABASE_URL ?? "postgres://postgres:postgres@127.0.0.1:5432/marketplace" });

app.get("/healthz", (_req, res) => res.json({ ok: true }));
app.get("/events", async (req, res) => {
  const limit = Math.min(Number(req.query.limit ?? 50), 200);
  const result = await db.query(
    `select signature, slot, log_index, event_kind, payload, finalized, block_time
     from market_events order by slot desc, log_index desc limit $1`,
    [limit],
  );
  res.json({ items: result.rows });
});

const port = Number(process.env.PORT ?? 8080);
app.listen(port, () => console.log(`api listening on :${port}`));
