import { PublicKey } from "@solana/web3.js";

export const PROGRAM_ID = new PublicKey("BDGYk9AwrWG8wKey93VzpetV8d2qhMoEbPyi89FJ9eCP");
export const METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

export function marketPda(programId = PROGRAM_ID): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from("market")], programId);
}

export function listingPda(assetMint: PublicKey, programId = PROGRAM_ID): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([Buffer.from("listing"), assetMint.toBuffer()], programId);
}

export function metadataPda(assetMint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), assetMint.toBuffer()],
    METADATA_PROGRAM_ID,
  );
}

export function splitPrice(price: bigint, feeBps: bigint) {
  if (feeBps < 0n || feeBps > 1_000n) throw new Error("fee outside safety cap");
  const fee = (price * feeBps) / 10_000n;
  return { sellerAmount: price - fee, fee };
}
