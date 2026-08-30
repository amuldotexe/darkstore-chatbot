# D08 — V001 Confido-Baseline Design v01

## Decision

Build the first dark-store fashion chatbot by adapting the **shape** of the
local Confido Tauri application, not its healthcare/prompt-analysis domain.
V001 is a local developer demonstration: it asks for a GPT-4o API key in a
Confido-style setup screen, retains it only in browser memory for the current
session, then lets GPT-4o pick three catalog-backed looks from bundled sample
data.

## Why this baseline

Confido already proves the right desktop seams: its webview accepts a typed
Tauri `invoke` function, its Rust layer centrally registers commands, and its
session-key form explicitly says the key is memory-only. The dark-store app
keeps those seams while replacing its domain model and entire workbench.

## V001 experience

1. App launch shows only the session-key gate.
2. The shopper enters a GPT-4o key and selects **Unlock my first edit**.
3. The Rust command validates the key with GPT-4o without logging or storing it.
4. The main workbench presents three looks chosen from the fixture catalog,
   each with price, available sizes, inventory count, delivery promise, and a
   GPT-supplied rationale.
5. The right rail displays the evidence used to ground the cards: fixture
   version, stock status, and the model label.

## Boundary design

| Boundary | Owns | Does not own |
| --- | --- | --- |
| Webview UI | password-field state, loading/error states, selected product detail, rendering | SKU truth, price, stock, model parsing |
| Typed Tauri commands | request validation and serializable errors | layout and DOM behavior |
| Rust core | fixture catalog, GPT request, response parsing, SKU allow-list validation | Tauri setup or browser state |
| Fixture data | products, variants, availability, delivery, look metadata | UI markup or model credentials |

`validate_session_api_key` and `curate_catalog_home_looks` are the only two
V001 commands. The application ships an embedded fixture file, so it requests
no filesystem permissions. It makes the OpenAI request from Rust rather than
the webview; this is appropriate for a developer-only local demo but must be
replaced by a server-held key before distribution.

## Security constraints

- No secret is committed, logged, stored in Tauri state, or saved to disk.
- UI failures must never echo the submitted key.
- Model responses can nominate only SKUs already present in the fixture.
- Malformed, duplicated, or unknown model choices return a user-visible error
  rather than producing invented merchandise.
- The Tauri capability stays limited to the `main` window and `core:default`.

## Out of scope

- Real inventory, payments, cart persistence, authentication, analytics,
  remote image loading, streaming, embeddings, and external product APIs.
- Any production key handling or key sharing.
