# Specs v001 — Dark-store fashion concierge

**Status:** Canonical v001 executable specification; three Tauri review iterations complete, with the v001 brief-search, final-inventory, and identity corrections implemented.
**Canonical journey:** [darkstore-propensity-cart-journey-v001.drawio](diagrams/darkstore-propensity-cart-journey-v001.drawio)
**Canonical architecture:** [turso-dress-architecture-v001.drawio](diagrams/turso-dress-architecture-v001.drawio)
**Historical inputs:** [archived problem statement](archive/prd-executable-specs/D01-problem-statement.md), [research precedents](D02-conversational-fashion-precedents.md), and [captured Slikk source rows](D05-slikk-dresses-sample-v01.csv).
**Runtime inventory:** [Turso/libSQL seed dump](../data/darkstore-dresses-v001.sql), with exactly eight dress rows.

## DMG Delivery Amendment — Embedded Catalogue

`REQ-TAURI-017.0` supersedes the remote-runtime assumptions in this document for the release DMG. The packaged v001 application SHALL compile an eight-product JSON projection of the checked-in source-compatible SQL seed into its Rust binary, derive the sole `dresses` taxonomy locally, and start without `TURSO_DATABASE_URL`, `TURSO_AUTH_TOKEN`, a dotenv file, or any startup catalogue network request. The existing Turso SQL dump remains the data-source reference; the remote adapter is not the packaged composition root. GPT-4o still requires a shopper-provided runtime key and still classifies category only.

**Verification:** `TEST-RUST-UNIT-EMBEDDED-017` loads the actual embedded repository, asserts one `dresses` taxonomy entry and eight source-derived product records, and does so without Turso configuration. Release verification mounts and launches the generated DMG before inspecting its key gate.

`REQ-TAURI-018.0` clarifies pagination continuity: browser re-renders SHALL retain the submitted search brief in application state, so the fourth `Show 3 more` card requests the next page using the same brief rather than the empty, freshly rendered input.

`REQ-TAURI-019.0` hardens the live OpenAI boundary: the Rust gateway SHALL map provider HTTP failures to shopper-safe typed errors without serializing the API key, request body, provider body, or transport internals. `401` SHALL report key rejection; `403` or `404` SHALL report GPT-4o project-access denial; `429` SHALL report project rate limiting; other `4xx` responses SHALL report a rejected request; and `5xx` or transport failures SHALL report temporary unavailability.

`REQ-TAURI-020.0` validates the GPT-4o Structured Outputs request itself: the `text.format.schema` SHALL be a single root object with `additionalProperties: false`, all four decision fields required, and no root union. The model SHALL return the discriminator `kind` plus nullable inactive-branch fields; Rust SHALL consume only a contract-valid `matched` or `not_in_inventory` decision.

`REQ-TAURI-021.0` validates the raw Responses REST result: after a successful provider response, the Rust gateway SHALL extract the JSON decision from the first non-empty `output[]` message content item whose type is `output_text`. The legacy top-level `output_text` convenience field remains accepted for fixture compatibility. Missing, blank, refusal-only, or malformed output SHALL remain a typed model-contract failure.

`REQ-TAURI-022.0` makes release packaging deterministic: `pnpm build:dmg:clean` SHALL remove Darkstore debug bundles and prior release bundle directories, clean the `darkstore-concierge` Cargo package, build one unsigned DMG, verify its disk-image checksum, and leave exactly one artifact at `src-tauri/target/release/bundle/dmg/Darkstore-Concierge-v001-current.dmg`.

## V001 Interaction Correction — Brief Search, Exhaustion, and App Identity

The search control exists to submit a new shopper brief, not to promise a
personally selected number of items. V001 still interprets the brief as a
category only, then ranks the available dresses deterministically. Therefore a
brief such as `black dress` can legitimately return the same high-propensity
dress set as the first look; the screen must make that successful result visible
instead of appearing inert.

The eight-dress fixture produces non-overlapping result pages of three, three,
then two dresses. V001 renders that final partial page honestly and follows it
with a terminal `No more inventory.` state. That terminal state is neither an
inventory-transport, model, nor generic command failure.

The desktop app is named **Darkstore Concierge**. `slikk/edit` is neither an
app name nor a permitted shopper-facing wordmark. Source-derived brand metadata
inside a product card remains product data, not application identity.

## Product Outcome and Boundaries

The v001 desktop app proves one narrow hypothesis: a seeded shopper context, trend snapshot, inventory classification data, and dark-store catalog can surface three grounded fashion choices that a shopper can customize with fixture-grounded chat guidance and add to a local cart faster than starting with unstructured search.

The shopper enters their own OpenAI API key at runtime. The release app reads an embedded eight-dress JSON projection compiled from the Turso-compatible source seed; it uses artificial persona, trend, availability, size, delivery, and propensity fixtures. It does not implement login, persisted preferences, checkout, payments, inventory synchronization, sidecars, or user-filesystem-backed product storage.

## V001 Embedded Inventory Boundary

The source reference is the `inventory_products` table in `data/darkstore-dresses-v001.sql`. It has exactly eight rows—well below the 100-row limit—and its only supported `category_id` is `dresses`. The release binary reads the equivalent checked-in JSON projection locally, with no runtime database network dependency. Source-derived fields retain the captured Slikk SKU, brand, title, price, URL, capture time, and merchandising data. The explicitly named `fixture_*` fields are v001 demo assumptions: availability, two or three size options, 50-minute delivery, deterministic propensity score, dress type, and style tags.

`TURSO_DATABASE_URL` and `TURSO_AUTH_TOKEN` may configure a development-only source adapter, but are not required for the packaged v001 release. The checked-in `.env.example` contains no values. The WebView never receives database credentials or talks to a database directly.

## V001 Category Then Propensity Decision Rule

GPT-4o receives the complete runtime taxonomy, which in v001 is the one-element list `[dresses]`. For the first look it uses seeded shopper persona and trend context; for “I want something else” it uses the retained free-text brief. It returns either a matched `dresses` category with short rationale or `not_in_inventory` with a shopper-safe acknowledgement. It never returns product IDs, product scores, prices, stock, or other commerce facts, and it never ranks products.

The Rust core validates the category result against Turso, filters to `fixture_available = 1` rows in that category, and deterministically takes the three highest fixture propensity scores. It breaks equal scores by ascending SKU. A `not_in_inventory` result renders no cards and immediately tells the shopper that this demo currently carries dresses only. This keeps GPT responsible for interpreting *what type* of product is wanted and keeps the app responsible for *which three* products are surfaced.

## Tauri Work Mode

- **Spec Mode** — define measurable v001 behavior before implementation.
- **App Architecture Mode** — define frontend, IPC, Rust-core, and session-state boundaries.
- **Desktop Security Mode** — constrain a user-supplied API key and desktop capabilities.
- **Lifecycle Mode** — define bounded startup and session cleanup.
- **Review Mode** — two recorded rubber-duck passes harden the packet.

## Executable Requirements

### REQ-TAURI-001.0: Configure session OpenAI key

**WHEN** the main window submits an API key
**THEN** the application SHALL validate it before enabling concierge actions
**AND** SHALL retain it only in in-memory session state
**SHALL** return the shopper to the key field with a serializable validation error when validation fails, without persisting or logging the key.

### REQ-TAURI-002.0: Generate grounded category trio

**WHEN** a valid session key, a matched category, and the Turso `inventory_products` table contain at least three `fixture_available = 1` rows in that category
**THEN** the application SHALL show exactly three distinct, fixture-available, displayable product cards plus one non-product “I want something else” action
**AND** SHALL render database-rehydrated facts only for the three highest-propensity products in that category
**SHALL** return a typed inventory-configuration error rather than render fewer than three product cards.

### REQ-TAURI-003.0: Classify inventory product category

**WHEN** GPT-4o receives the supplied runtime taxonomy `[dresses]` with either seeded first-look context or a shopper free-text brief
**THEN** the application SHALL accept either a `matched` result containing exactly `dresses` and a short rationale, or a `not_in_inventory` result with a shopper-safe acknowledgement
**AND** SHALL reject blank, multiple, malformed, unsupported, or product-ID results before product selection
**SHALL** never silently substitute `dresses` for a shopper request that does not match the runtime inventory.

### REQ-TAURI-004.0: Search portfolio pages

**WHEN** a shopper submits a free-text brief or selects the fourth initial card
**THEN** the application SHALL return up to three eligible search-result cards,
with exactly three cards on every non-final page and one or two cards permitted
only on the final page
**AND** SHALL classify the brief to matched `dresses`, rank only that category by propensity, and exclude SKUs already shown for the same brief and result stream
**SHALL** show `Show 3 more` while any unseen eligible dress remains; after the
final page it SHALL render `No more inventory.` and use absent-category recovery
only when the brief is not a dress request.

### REQ-TAURI-005.0: Re-anchor selected product chat

**WHEN** a shopper selects a first-look, search-result, or alternative-product card
**THEN** the application SHALL enter the same product-anchored chat state with the selected product ID, selection source, and retained brief
**AND** SHALL preserve that state when the shopper opens and returns from the optional product-detail view
**SHALL** not convert a product-detail view into a second chat or a new ungrounded recommendation request.

### REQ-TAURI-006.0: Confirm purchasable variant

**WHEN** the shopper requests add-to-cart from product-anchored chat
**THEN** the application SHALL require a selectable available size, colour, or variant when the product has variant choices
**AND** SHALL carry the selected variant through the cart request
**SHALL** keep add-to-cart unavailable until required variant information is valid.

### REQ-TAURI-007.0: Recheck catalog before cart

**WHEN** the shopper confirms a product and variant for local cart
**THEN** the application SHALL re-read price and availability from Turso immediately before the cart mutation
**AND** SHALL add only the revalidated product and variant
**SHALL** preserve the shopper brief and offer three fixture-available alternatives only when a complete alternative set exists; otherwise it SHALL retain chat context and invite the shopper to revise the brief.

### REQ-TAURI-008.0: Continue local cart journey

**WHEN** a revalidated product is added to cart
**THEN** the application SHALL update only the in-memory v001 cart and visible cart count
**AND** SHALL offer return actions for first look, a new free-text brief, and alternatives without discarding cart or chat context
**SHALL** omit checkout, payment, and persistent-cart behavior.

### REQ-TAURI-009.0: Ignore stale recommendation response

**WHEN** a second first-look, search, or alternative request supersedes an earlier request in the same session
**THEN** the application SHALL mark the earlier request stale before starting the newer request
**AND** SHALL allow only the latest request to mutate visible recommendation, pagination, or chat-anchor state
**SHALL** discard a late success or failure from a stale request without replacing the shopper’s newer intent or showing a false error.

### REQ-TAURI-010.0: Return typed command errors

**WHEN** a frontend-visible command encounters invalid input, missing Turso state, Turso or model transport failure, malformed model output, or unavailable product state
**THEN** the command SHALL return an owned serializable Result<T, AppError> boundary with a stable error kind and shopper-safe message
**AND** SHALL keep I/O-heavy command work asynchronous with owned inputs
**SHALL** not panic, log the API key, or leak provider response bodies to the frontend.

### REQ-TAURI-011.0: Scope desktop capability and CSP

**WHEN** the v001 desktop bundle configures its main window
**THEN** the main window label SHALL receive only the application commands required by this specification
**AND** SHALL grant no filesystem, shell, sidecar, plugin-store, updater, external-binary, or remote-webview image permission
**SHALL** use an explicit CSP with connect-src limited to self because model and Turso traffic originate in the Rust gateway, not the webview.

### REQ-TAURI-012.0: Bound startup and session cleanup

**WHEN** the application starts, replaces the API key, or closes the main window
**THEN** startup SHALL enter a key-required state without issuing a model request or restoring a historical secret
**AND** replacing or closing the session SHALL clear the in-memory secret, chat context, and active request identifiers
**SHALL** prevent an in-flight or late model response from mutating a cleared or closed session.

### REQ-TAURI-013.0: Rank validated category propensity

**WHEN** Rust core receives one validated category ID for a first-look, search, pagination, or alternative request
**THEN** it SHALL query Turso for distinct `fixture_available = 1` products in that category and sort them by `fixture_propensity_score DESC, sku ASC`
**AND** SHALL resolve equal propensity scores by ascending SKU before applying shown-SKU exclusions and taking at most the next three
**SHALL** require a three-card set for first look, but SHALL return the final one
or two eligible dresses for a later search page instead of crossing into another
category or hiding remaining inventory.

### REQ-TAURI-014.0: Recover absent inventory category

**WHEN** GPT-4o returns `not_in_inventory`, or a purported matched category has no row in Turso
**THEN** the application SHALL render no product cards and tell the shopper that v001 currently carries dresses only
**AND** SHALL retain and refocus the brief so the shopper can revise it without losing session or cart context
**SHALL** not substitute another category, issue a product-ranking query, or make a second model call to conceal the unavailable category.

### REQ-TAURI-015.0: Distinguish unavailable inventory transport

**WHEN** Turso configuration is absent, taxonomy lookup fails, or a Turso query times out or returns a malformed inventory row
**THEN** the application SHALL return the stable `inventory_unavailable` error kind with shopper-safe retry copy
**AND** SHALL preserve the shopper’s key, brief, chat anchor, and local cart while showing neither product cards nor a `not_in_inventory` dresses-only message
**SHALL** not call GPT-4o when the taxonomy cannot be read, because the runtime category list is unavailable.

### REQ-TAURI-016.0: Surface final inventory page

**WHEN** one or two eligible unseen `dresses` remain in a search or alternative result stream after shown-SKU exclusions
**THEN** the application SHALL return those remaining cards as the final page,
hide `Show 3 more`, and render `No more inventory.`
**AND** SHALL preserve the brief and chat context without switching categories
or fabricating cards
**SHALL** return `complete_page_exhausted` only when no eligible unseen dress
remains or a duplicate next-page request bypasses the visible terminal state.

### REQ-TAURI-018.0: Retain pagination brief

**WHEN** a shopper submits a free-text brief that produces a three-card page with a `Show 3 more` action
**THEN** the application SHALL keep the normalized brief in application state and render it back into the search field
**AND** SHALL send that same retained brief with `show_next_page = true` when the shopper selects `Show 3 more`
**SHALL** not read an empty replacement input after the first result-page re-render.

### REQ-TAURI-023.0: Submit an alternate brief visibly

**WHEN** a shopper enters a non-blank alternate brief and selects the search
submit control
**THEN** the control SHALL be labelled `Show matching dresses`, become disabled
while its request is pending, and invoke `search_portfolio_products_page` once
with the normalized brief and `show_next_page = false`
**AND** a successful response SHALL replace the prior tray with exactly the
returned three cards, identify that it is a result for the submitted brief, and
state that v001 ranks available dresses by category and propensity
**SHALL** not leave the previous first-look tray as the only visible outcome of
a completed brief submission, even when the returned SKUs are identical.

### REQ-TAURI-024.0: End a depleted discovery stream clearly

**WHEN** the current search result page has no additional unseen dress after
any remaining one- or two-dress final page has been shown
**THEN** the application SHALL remove the `Show 3 more` action and render the
shopper-facing terminal copy `No more inventory.` below the current valid tray
**AND** a guarded `complete_page_exhausted` response from a stale, repeated, or
otherwise invalid next-page invocation SHALL resolve to that same terminal
state while preserving the last valid tray and retained brief
**SHALL** not hide remaining eligible inventory, render a generic transport
error, or render an absent-category message for this condition.

### REQ-TAURI-025.0: Use Darkstore Concierge identity

**WHEN** any shopper-facing v001 screen, window title, startup screen, or
discovery header renders application identity
**THEN** it SHALL identify the app as `Darkstore Concierge`
**AND** it SHALL not render `slikk/edit`, `Slikk edit`, or a variation as the
application wordmark or title
**SHALL** preserve source-derived product-brand fields only within product
metadata, where they are not presented as the application identity.

## Tauri Design (Frontend/IPC/Rust Core + Permissions/Lifecycle)

### Frontend Contract

The TypeScript frontend owns rendering and ephemeral screen state. It invokes narrow typed operations for session setup, first look, search paging, product-chat selection, variant confirmation, and local cart mutation. Product-chat replies are explicitly fixture-grounded v001 guidance; GPT-4o remains category-only. A category response is either `matched` with `dresses` and rationale, `not_in_inventory` with shopper-safe copy, or `inventory_unavailable` with retry copy; the latter two render no cards but are visibly distinct. Product cards render only Turso-rehydrated fields.

Every recommendation-producing action carries a frontend-generated request ID. The reducer records the latest request ID for its stream and ignores all result or error actions whose ID no longer matches. The frontend receives a normalized tagged success-or-error DTO, never an opaque rejected promise or raw provider payload.

The alternate-brief screen owns three explicit visual states: `BriefEntry`,
`BriefPending`, and `BriefResults`. `BriefEntry` shows no stale first-look tray;
`BriefPending` disables `Show matching dresses` and exposes progress; and
`BriefResults` visibly labels the returned set with the submitted brief and
v001's category-then-propensity limit. `show_next_three = false` after a valid
search page renders the terminal `No more inventory.` notice. A
`complete_page_exhausted` command error is normalized to the same terminal
presentation only for an attempted subsequent page; it is not shown as a
generic error. App-level copy uses `Darkstore Concierge`; product brands remain
within card metadata.

### Command Layer

Tauri commands validate owned request DTOs, return Result<T, AppError>, and delegate Turso category validation, propensity ranking, pagination, and cart checks to Rust core services. Database and model commands are async and use owned request inputs. The GPT gateway returns only a category decision; no command receives a full product object from the frontend as authority.

The frontend-visible command identifiers are `configure_session_openai_key`, `load_initial_product_trio`, `search_portfolio_products_page`, `select_product_chat_context`, `update_product_variant_selection`, `add_validated_variant_cart`, and `clear_session_secret_state`. Category classification, Turso queries, validation, and ranking remain internal Rust operations rather than frontend-callable authority. Registration remains centralized in one command module.

### Rust Core and Managed State

Rust core owns a libSQL/Turso client, taxonomy lookup from `SELECT DISTINCT category_id FROM inventory_products`, category-ID validation, database eligibility, deterministic within-category propensity ordering, pagination, selection context, variant validation, local-cart mutation, and session-generation checks. It queries `inventory_products` by category and availability, ordered by `fixture_propensity_score DESC, sku ASC`; all eight v001 rows are `dresses`. Turso credentials are read only from backend process configuration and are never stored in Tauri managed state. Tauri manages one session state object containing the runtime OpenAI key, current chat context, retained briefs, shown-SKU streams, selected variant, and local cart. The TypeScript reducer owns latest-request IDs; the Rust session generation prevents a cleared or replaced session from receiving a late commit. Commands copy required state briefly, then release synchronization before any database or model await.

### Permissions and Lifecycle

v001 uses one main window with the stable label main. Its capability file permits only the commands in this packet and deliberately grants no filesystem, shell, sidecar, updater, plugin-store, external-binary, or remote-webview image permission. Product images are bundled local assets; adding a remote image host is a later, separately scoped security decision.

The WebView CSP is explicit and keeps connect-src at self. The Rust gateways, rather than the WebView, perform HTTPS only to configured OpenAI and Turso endpoints. Both adapters are trait-backed for tests and never expose the OpenAI key, Turso token, provider body, or raw transport error over IPC.

Startup transitions directly to KeyRequired and starts no network work. The clear_session_secret_state operation invalidates current request IDs, clears key, chat context, shown-ID streams, and cart state, then prevents late work from committing.

## Verification Matrix

| req_id | test_id | test_type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-TAURI-001.0 | TEST-RUST-UNIT-KEY-001 | rust-unit | invalid key keeps session in key-required state and exposes no secret | key handling |
| REQ-TAURI-002.0 | TEST-RUST-UNIT-TRIO-002 | rust-unit | matched dresses query renders its three highest-propensity distinct cards plus fourth action | first look |
| REQ-TAURI-002.0 | TEST-SEED-SQL-INVENTORY-002 | sql-seed | one strict `inventory_products` table has exactly eight available `dresses` rows and 2–3 sizes each | bounded inventory |
| REQ-TAURI-003.0 | TEST-RUST-UNIT-CATEGORY-003 | rust-unit | only `matched:dresses` or `not_in_inventory` is accepted; malformed, multiple, unsupported, and product-ID values recover safely | model grounding |
| REQ-TAURI-004.0 | TEST-RUST-UNIT-PAGE-004 | rust-unit | dress-ranked pages exclude shown SKUs, return 3/3/2 products from the eight-dress fixture, and expose `Show 3 more` while an unseen dress remains | search recovery |
| REQ-TAURI-005.0 | TEST-FRONTEND-CHAT-005 | frontend | every card source produces one retained chat state | selection handoff |
| REQ-TAURI-006.0 | TEST-FRONTEND-VARIANT-006 | frontend | add control remains disabled until a required variant is selected | variant gate |
| REQ-TAURI-007.0 | TEST-RUST-INTEG-STOCK-007 | rust-integration | changed Turso availability routes to alternatives without cart mutation | commerce truth |
| REQ-TAURI-008.0 | TEST-FRONTEND-CART-008 | frontend | local cart persists while discovery route changes | continuation |
| REQ-TAURI-009.0 | TEST-RUST-UNIT-RACE-009 | rust-unit | only the latest request ID can commit recommendation state | stale-response safety |
| REQ-TAURI-010.0 | TEST-RUST-INTEG-ERROR-010 | rust-integration | command returns serializable AppError for Turso and model failures without credential leakage | command boundary |
| REQ-TAURI-019.0 | TEST-RUST-UNIT-OPENAI-019 | rust-unit | provider HTTP statuses produce shopper-safe typed errors with no provider body or credential exposure | live-model diagnostics |
| REQ-TAURI-020.0 | TEST-RUST-UNIT-OPENAI-020 | rust-unit | request schema is one strict root object and a full strict match payload parses to a category decision | provider request validity |
| REQ-TAURI-021.0 | TEST-RUST-UNIT-OPENAI-021 | rust-unit | raw Responses `output[]` message content extracts the first valid `output_text` JSON decision | provider response validity |
| REQ-TAURI-022.0 | TEST-SHELL-DMG-CLEAN-022 | shell | clean-only release mode removes stale `dmg` and `macos` package directories | release determinism |
| REQ-TAURI-011.0 | TEST-CONFIG-CAPABILITY-011 | config | main capability has required commands and no filesystem, shell, plugin, sidecar, updater, remote-image, or direct Turso grant | least privilege |
| REQ-TAURI-012.0 | TEST-RUST-UNIT-LIFECYCLE-012 | rust-unit | clear session invalidates key, context and request IDs; late response cannot commit | lifecycle safety |
| REQ-TAURI-013.0 | TEST-SEED-SQL-PROPENSITY-013 | sql-seed | available dresses are ordered by score descending, then SKU ascending; equal-score rows prove the tie-break | deterministic ranking |
| REQ-TAURI-013.0 | TEST-RUST-UNIT-PROPENSITY-013 | rust-unit | only eligible Turso rows in the validated category are ordered by descending score, then ascending SKU, with a final partial page permitted after first look | deterministic ranking |
| REQ-TAURI-014.0 | TEST-FRONTEND-ABSENT-014 | frontend | a shirts, jacket, or shoes brief displays Dresses-only recovery, no cards, and an editable brief | absent-category recovery |
| REQ-TAURI-015.0 | TEST-RUST-INTEG-INVENTORY-015 | rust-integration | absent config, timeout, or malformed row returns `inventory_unavailable`, preserves context, and does not call the model before taxonomy lookup | transport truth |
| REQ-TAURI-016.0 | TEST-RUST-UNIT-PAGESET-016 | rust-unit | one or two remaining rows return as the final page; a zero-row repeated request returns complete-page-exhausted | final inventory page |
| REQ-TAURI-018.0 | TEST-FRONTEND-PAGINATION-018 | frontend | next-three request retains the submitted brief instead of reading a re-rendered empty field | pagination continuity |
| REQ-TAURI-023.0 | TEST-FRONTEND-BRIEF-023 | frontend | `Show matching dresses` disables during one typed invoke, then replaces the stale tray and labels the submitted brief even when returned SKUs match first look | brief submission |
| REQ-TAURI-023.0 | TEST-MOCK-TAURI-BRIEF-023 | mocked-tauri | mock `search_portfolio_products_page` receives normalized `black dress` and `showNextPage: false`; pending, success, and typed-error UI states are distinct | IPC/UI contract |
| REQ-TAURI-024.0 | TEST-RUST-UNIT-EXHAUSTION-024 | rust-unit | an eight-dress stream yields non-overlapping 3/3/2 pages; a zero-row repeated request is a typed complete-page-exhausted guard | core pagination |
| REQ-TAURI-024.0 | TEST-FRONTEND-EXHAUSTION-024 | frontend | final valid one- or two-card page and guarded repeated-next error hide `Show 3 more`, retain the valid tray and brief, and show `No more inventory.` | terminal recovery |
| REQ-TAURI-025.0 | TEST-FRONTEND-IDENTITY-025 | frontend | key gate, header, discovery, chat, and window-facing copy expose `Darkstore Concierge` and no app-identity `slikk/edit` string | app identity |

## Implementation Plan

### TDD Plan

1. **STUB** — add frontend state and mocked-Tauri boundary tests for REQ-TAURI-023.0 through REQ-TAURI-025.0 before changing the brief form, terminal pagination copy, or app identity. Retain the existing typed tests for category validation, complete-page exhaustion, stale-response, AppError, capability, CSP, and session-clear cases.
2. **RED** — run Rust-core and frontend contract tests; record missing command and reducer surfaces.
3. **GREEN** — implement the explicit brief-entry/pending/results state, shopper-visible terminal exhaustion path, and Darkstore Concierge identity with the minimum frontend and typed-command changes. Preserve category-only model output validation, within-category propensity order, selection state, variant gate, and local-cart mutation.
4. **REFACTOR** — isolate the model gateway, shrink command handlers, and preserve four-word names for new internal surfaces.
5. **VERIFY** — run Rust, frontend, mocked-Tauri, capability/CSP configuration, and static spec-traceability checks before any application commit.

## Quality Gate Results

- [x] Requirements use stable REQ-TAURI identifiers.
- [x] Requirements REQ-TAURI-001.0 through REQ-TAURI-016.0 map to at least one planned test.
- [x] Turso seed dump is a one-table, eight-row, `dresses`-only catalogue with two or three sizes per product.
- [x] Empty, invalid, unavailable, and exhausted-result states have explicit behavior.
- [x] Tauri App Architecture review — iteration 1 added request freshness and typed command-error contracts.
- [x] Tauri Desktop Security and Lifecycle review — iteration 2 added a main-label capability, restrictive CSP, and bounded session lifecycle.
- [x] Rust and frontend build gates — `cargo fmt`, Clippy with warnings denied, Rust tests/build, Vitest, TypeScript, and Vite production build pass locally with deterministic adapters.
- [x] REQ-TAURI-023.0 brief submission state is implemented and verified at the frontend/Tauri boundary.
- [x] REQ-TAURI-024.0 final partial inventory page and terminal `No more inventory.` copy are implemented and verified against the eight-dress fixture.
- [x] REQ-TAURI-025.0 Darkstore Concierge identity replaces all shopper-facing `slikk/edit` app branding and is verified.

## Open Questions

1. Which user-owned Turso organization, database URL, and scoped auth token will configure the backend locally?
2. Should a later version add a separate, product-grounded GPT response for customization chat, or retain the deliberately deterministic fixture guidance used in v001?
3. Should the optional product-detail screen be a route within the main window or a separate label-scoped window?
4. Are bundled local product images sufficient for the demo, or should a future version add a separately scoped remote-image policy?

## Rubber-Duck Review Log

The next two sections record the required Tauri executable-spec reviews. A resolution is accepted only when it becomes a requirement, a design boundary, and a traceability row.

### Iteration 1 — App Architecture Review

**Rubber-duck trace:** “I submit a linen-shirt brief, then immediately request a black dress. The linen response fails or returns late. Which state may it overwrite, and how does the UI receive that failure?”

**Gap found:** the first pass had neither a latest-request ownership rule nor a typed frontend-visible failure boundary. A late answer could replace the newer intent; a provider failure could degrade into an opaque invoke rejection.

**Resolution:** REQ-TAURI-009.0 makes request freshness explicit across first look, search, and alternatives. REQ-TAURI-010.0 makes command errors serializable and requires owned async inputs. The frontend reducer, command identifiers, and verification matrix now carry both contracts.

### Iteration 2 — Desktop Security and Lifecycle Review

**Rubber-duck trace:** “I paste a key, close the app while a model request is running, and later reopen it. Which window can invoke what? Does the key return? Can the previous response populate the new session? Does the WebView have a direct network escape hatch?”

**Gap found:** the first pass described session-only handling, but it did not make the capability, CSP, main-window label, or clear-on-close behavior executable.

**Resolution:** REQ-TAURI-011.0 fixes the main capability and CSP boundary: no filesystem, shell, plugins, sidecars, updater, remote image, or WebView model traffic. REQ-TAURI-012.0 defines KeyRequired startup and invalidates secret, context, and active request IDs at session replacement or close. The configuration and lifecycle tests now trace both decisions.

### Assumption refinement — Category then propensity

**Rubber-duck trace:** “If GPT returns product IDs, who guarantees that the three initial cards are genuinely the highest-propensity products within the intended type? What happens when it emits an unavailable or cross-category product?”

**Gap found:** the first v001 packet let the model rank catalog candidates. That mixed language interpretation with product ranking, obscured the category taxonomy, and weakened reproducibility.

**Resolution:** GPT-4o now returns one validated category only under REQ-TAURI-003.0. REQ-TAURI-013.0 makes Rust filter to that category and rank by fixture propensity score, with a product-ID tie-break. REQ-TAURI-002.0 and REQ-TAURI-004.0 make the first trio and free-text pages consume that same category-first pipeline.

### Data-boundary refinement — Turso, eight dresses, and no substitution

**Rubber-duck trace:** “The shopper asks for a linen shirt. The current runtime taxonomy contains only dresses. Could we silently show dress cards? Where do source facts end and v001 demo assumptions begin? Can the WebView see the Turso token?”

**Gap found:** the category-first packet did not name a concrete runtime inventory store, a bounded row count, or a first-class absent-category result. It also did not distinguish source-derived Slikk data from fixture values.

**Resolution:** `data/darkstore-dresses-v001.sql` seeds one strict Turso-compatible table with eight `dresses` rows and two or three size options each. REQ-TAURI-014.0 forbids substitute cards for an unsupported request and restores the editable brief. The Rust-only Turso client reads local environment credentials; the WebView receives only rehydrated DTOs.

### Iteration 3 — Inventory truth and complete-page review

**Rubber-duck trace:** “The seed has eight dresses. After two three-card pages, two rows remain. Does the UI show them despite its three-card promise? Separately, a Turso timeout happens before taxonomy lookup. Does the shopper see ‘we only carry dresses’ even though the app simply cannot read the database? Finally, is a seeded `fixture_available` value being presented as live stock?”

**Gaps found:** the packet used live-stock language for fixture data, did not distinguish an unavailable database from a valid absent-category answer, and made the fourth card unconditional without stating how a final partial inventory page should behave.

**Resolution:** REQ-TAURI-015.0 introduces the typed `inventory_unavailable` state and blocks the model call until taxonomy is available. REQ-TAURI-016.0 now renders a final one- or two-card page, followed by `No more inventory.` and no fourth continuation action; only a zero-row repeat returns `complete_page_exhausted`. REQ-TAURI-002.0, -004.0, -007.0, and -013.0 now use `fixture_available` language; the architecture diagram separates source facts, demo fixtures, model interpretation, and Turso transport truth.
