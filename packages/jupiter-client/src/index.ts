import { Connection, Keypair, VersionedTransaction } from "@solana/web3.js";

export type QuoteRequest = {
  inputMint: string;
  outputMint: string;
  amount: bigint;
  slippageBps?: number;
};

export type JupiterClientOptions = {
  baseUrl?: string;
  apiKey?: string;
};

export class JupiterClient {
  private readonly baseUrl: string;
  private readonly apiKey?: string;

  constructor(options: JupiterClientOptions = {}) {
    this.baseUrl = options.baseUrl ?? "https://lite-api.jup.ag/swap/v1";
    this.apiKey = options.apiKey;
  }

  private headers(): HeadersInit {
    return this.apiKey ? { "x-api-key": this.apiKey } : {};
  }

  async quote(request: QuoteRequest): Promise<unknown> {
    const url = new URL(`${this.baseUrl}/quote`);
    url.searchParams.set("inputMint", request.inputMint);
    url.searchParams.set("outputMint", request.outputMint);
    url.searchParams.set("amount", request.amount.toString());
    url.searchParams.set("slippageBps", String(request.slippageBps ?? 50));
    const response = await fetch(url, { headers: this.headers() });
    if (!response.ok) throw new Error(`Jupiter quote failed: ${response.status}`);
    return response.json();
  }

  async buildSwap(quoteResponse: unknown, userPublicKey: string): Promise<VersionedTransaction> {
    const response = await fetch(`${this.baseUrl}/swap`, {
      method: "POST",
      headers: { "content-type": "application/json", ...this.headers() },
      body: JSON.stringify({ quoteResponse, userPublicKey, dynamicComputeUnitLimit: true }),
    });
    if (!response.ok) throw new Error(`Jupiter swap build failed: ${response.status}`);
    const payload = (await response.json()) as { swapTransaction: string };
    return VersionedTransaction.deserialize(Buffer.from(payload.swapTransaction, "base64"));
  }

  async signAndSend(connection: Connection, tx: VersionedTransaction, signer: Keypair) {
    tx.sign([signer]);
    return connection.sendTransaction(tx, { maxRetries: 3 });
  }
}
