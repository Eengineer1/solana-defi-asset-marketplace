const endpoint = import.meta.env.VITE_API_URL ?? "http://localhost:8080";
const app = document.querySelector<HTMLElement>("#app")!;
app.innerHTML = `
  <style>body{font-family:ui-sans-serif,system-ui;max-width:980px;margin:48px auto;padding:0 24px;background:#0b0e14;color:#e6edf3}code{color:#9cdcfe}.card{border:1px solid #30363d;border-radius:12px;padding:20px;margin:16px 0}</style>
  <h1>Asset Marketplace</h1>
  <p>Reference operator surface for indexed Anchor marketplace events and Jupiter-assisted settlement preparation.</p>
  <section class="card"><strong>Program</strong><br><code>BDGYk9AwrWG8wKey93VzpetV8d2qhMoEbPyi89FJ9eCP</code></section>
  <section class="card"><strong>Indexer status</strong><pre id="status">loading…</pre></section>
`;
fetch(`${endpoint}/events?limit=10`)
  .then((r) => r.json())
  .then((x) => { document.querySelector("#status")!.textContent = JSON.stringify(x, null, 2); })
  .catch((e) => { document.querySelector("#status")!.textContent = String(e); });
