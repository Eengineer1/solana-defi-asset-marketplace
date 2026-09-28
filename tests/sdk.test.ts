import test from "node:test";
import assert from "node:assert/strict";
import { PublicKey } from "@solana/web3.js";
import { listingPda, marketPda, metadataPda, splitPrice } from "../packages/sdk/src/index.js";

test("market/listing/metadata PDA derivations are deterministic", () => {
  const mint = new PublicKey("So11111111111111111111111111111111111111112");
  assert.equal(marketPda()[0].toBase58(), marketPda()[0].toBase58());
  assert.equal(listingPda(mint)[0].toBase58(), listingPda(mint)[0].toBase58());
  assert.equal(metadataPda(mint)[0].toBase58(), metadataPda(mint)[0].toBase58());
});

test("market fee split preserves gross price", () => {
  const { sellerAmount, fee } = splitPrice(1_000_000n, 250n);
  assert.equal(fee, 25_000n);
  assert.equal(sellerAmount + fee, 1_000_000n);
});
