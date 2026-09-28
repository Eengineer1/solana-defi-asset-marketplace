# Jupiter routing integration

`packages/jupiter-client` is intentionally small and inspectable:

- configurable base URL and optional API key;
- quote requests bind input mint, output mint, amount and slippage;
- swap construction asks Jupiter for a versioned transaction with dynamic compute limit;
- signing and submission remain explicit client operations.

The default URL targets Jupiter's lite Swap API shape. Integrators can override `baseUrl` without changing the marketplace SDK.

The marketplace program itself does not CPI into Jupiter. Users can swap into the required payment mint first, then execute the deterministic marketplace `buy` instruction. This keeps DEX routing failure/slippage separate from asset settlement and removes arbitrary route accounts from the on-chain marketplace trust boundary.
