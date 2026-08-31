# Spec Improvement 02 — Grounded GPT Dress Selection and Shareable DMG

**Status:** Approved for TDD delivery. This supersedes the v001 category-classification rule for recommendation selection.

## Product decision

The shopper is already inside a dress-only demonstration store. GPT-4o should
choose the available dresses, not decide whether a colour, occasion, mood, or
style is a valid product category. For every non-empty brief, the app provides
the currently available, unseen dress records to GPT-4o and asks it to return
the best three SKU identifiers in ranked order. The Rust core validates that
answer before a card can be shown. If the model call fails or its response is
not a valid subset of the offered inventory, the app returns the locally
ranked propensity trio instead of a category-contract error.

This makes `red`, `black`, `wedding`, and `date night` useful preference
signals. It also removes the misleading expectation that a shopper must name
the word “dress” for the app to work.

## Tauri work mode

- Spec mode
- App architecture mode
- Desktop security and lifecycle mode

## Executable requirements

### REQ-TAURI-026.0: Select from the offered available inventory

**WHEN** an enabled session asks for its first look or submits any non-empty
brief
**THEN** the Rust workflow **SHALL** load the available, unseen dress records
before calling GPT-4o
**AND** the request **SHALL** contain only those candidate product facts and
SKU identifiers
**AND** GPT-4o **SHALL** return up to three SKU identifiers in shopper-ranked
order, never an arbitrary product or a category decision.

### REQ-TAURI-027.0: Validate GPT recommendations before rendering

**WHEN** GPT-4o returns recommended SKUs
**THEN** the Rust core **SHALL** accept only unique SKUs that were present in
that call’s available candidate set
**AND** SHALL require exactly three when three or more candidates were offered
**AND** SHALL preserve the validated model order in the returned product cards.

### REQ-TAURI-028.0: Fall back safely on an unusable model result

**WHEN** the GPT-4o request fails, cannot be parsed, contains a duplicate,
unknown, unavailable, or incomplete SKU list
**THEN** the workflow **SHALL** return the locally propensity-ranked available
dresses for that page
**AND** SHALL not expose a model-category or model-contract error to the
shopper
**AND** SHALL retain the same no-duplicate pagination behavior.

### REQ-TAURI-029.0: Treat preferences as preferences

**WHEN** a shopper submits a non-empty colour, fabric, fit, style, mood, or
occasion brief such as `red`, `black`, or `wedding`
**THEN** the application **SHALL** return up to three available dress cards
**AND** the call-to-action **SHALL** say `Show matching dresses`
**AND** the screen SHALL not render `The model response did not match the
category contract` or `Not in this demo` solely because the brief lacks a
category noun.

### REQ-TAURI-030.0: Keep the alternative path clear and finite

**WHEN** a recommendation page has unseen dresses
**THEN** its fourth card **SHALL** read `Show 3 more`
**AND WHEN** no unseen dresses remain
**THEN** the fourth card **SHALL** read `No more inventory.` and be
non-interactive
**AND WHEN** the shopper is at the first look
**THEN** the fourth card **SHALL** read `Search another dress` with explicit
dark foreground contrast.

### REQ-TAURI-031.0: Prevent duplicate cart actions in the visible UI

**WHEN** the shopper adds a validated selected size to the local cart
**THEN** the app **SHALL** show the updated cart count and an `Added to local
cart` disabled action for that same selected variant
**AND WHEN** the shopper selects another available size
**THEN** the action **SHALL** become available again for that variant.

### REQ-TAURI-032.0: Build a clean Developer ID-signed universal DMG

**WHEN** the release build runs on the signing Mac
**THEN** it **SHALL** remove old bundle output before compiling a fresh
`universal-apple-darwin` DMG
**AND** SHALL sign the application with the local Developer ID Application
identity
**AND** SHALL verify the mounted app with `codesign --verify --deep --strict`.

**Boundary:** Developer ID signing is distinct from Apple notarization. A
notarized share-to-any-Mac release additionally requires Apple notarization
credentials and a successful `spctl` assessment; this build must report that
separately rather than claiming it.

### REQ-TAURI-033.0: Keep API keys outside the release artifact

**WHEN** the app is packaged or its source is committed
**THEN** no OpenAI API key **SHALL** be embedded in the source, test fixture,
DMG, configuration, or build log
**AND** a shopper key SHALL remain session-only in the running app.

## Tauri design

| Area | Contract |
| --- | --- |
| Frontend | Existing typed `loadInitialProductTrio` and `searchPortfolioProductsPage` response remains a `RecommendationOutcome`; copy reflects dress recommendation rather than category classification. |
| Command layer | Existing command IDs stay stable. Commands remain thin and return serializable `AppError` only for inventory/session faults not covered by the safe ranking fallback. |
| Rust core | Replace the category-classification model port with a product-selection port. It receives an owned product candidate snapshot and brief, returns ranked SKU strings, validates them, and falls back to existing local propensity ranking. |
| Managed state | Existing session mutex remains the owner of API key, shown SKUs, retained brief, selected size, and cart. No lock is held across the model HTTP await. |
| Capabilities | No new filesystem, shell, plugin, or window capability is needed. The network boundary stays in the Rust adapter. |
| Lifecycle/package | The release script cleans only `src-tauri/target/release/bundle` output, builds the universal target, discovers the Developer ID identity, and verifies the produced app. |

## Verification matrix

| Requirement | Test ID | Level | Assertion |
| --- | --- | --- | --- |
| REQ-TAURI-026.0 | TEST-RUST-026 | Rust unit | Selection payload contains offered available SKU candidates and a strict array schema. |
| REQ-TAURI-027.0 | TEST-RUST-027 | Rust integration | Valid model order is rendered as cards; unknown, duplicate, and incomplete selections are rejected by the validator. |
| REQ-TAURI-028.0 | TEST-RUST-028 | Rust integration | A model error or invalid selection returns deterministic local cards without a model error. |
| REQ-TAURI-029.0 | TEST-FRONTEND-029 | Frontend | `black` submits and renders three cards; no absence/contract message appears. |
| REQ-TAURI-030.0 | TEST-FRONTEND-030 | Frontend | First, next, and terminal fourth-card labels and interaction state are correct. |
| REQ-TAURI-031.0 | TEST-FRONTEND-031 | Frontend | Cart action is disabled after a successful add and enabled by a different size. |
| REQ-TAURI-032.0 | TEST-PACKAGE-032 | Shell/desktop | Fresh output is one universal signed DMG; mounted app passes `codesign` verification. |
| REQ-TAURI-033.0 | TEST-PACKAGE-033 | Shell | Repository and release artifact contain no API key pattern or secret fixture. |

## TDD delivery sequence

1. **STUB:** Add the model-selection parser, validator, workflow fallback, and frontend-flow tests.
2. **RED:** Run the new focused tests against the current category contract and record the expected failures.
3. **GREEN:** Implement the narrow model selection port, validation, fallback, and visible button-state changes.
4. **REFACTOR:** Remove obsolete category-decision code only after the replacement path is green; retain stable command IDs.
5. **VERIFY:** Run Rust format, clippy, all Rust tests, frontend tests/build, clean signed universal package build, mount it, verify code signature, and capture the release evidence in the TDD journal.

## Non-goals

- Live product images, payment, checkout, or remote inventory writes.
- Inventing catalog records or showing products that were not offered by the Rust catalog snapshot.
- Persisting an OpenAI key, keychain integration, or transmitting an API key into frontend logs.
- Claiming Apple notarization without running Apple’s notarization flow successfully.
