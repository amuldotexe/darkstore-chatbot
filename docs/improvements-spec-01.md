# Improvements Spec 01 — Visible Alternate Discovery Card

## Observed defect

The fourth card in the first-look grid is the alternate-discovery action, not a
fourth product. `renderProductCardGrid()` renders it as the native
`<button class="fourth-card">` with the **I want something else** copy
([`src/app.ts`](../src/app.ts#L142-L149)).

The shared card rule gives it the cream card surface, but the later
`.fourth-card` rule changes that surface to `transparent` and sets no explicit
`color` ([`src/styles.css`](../src/styles.css#L45),
[`src/styles.css`](../src/styles.css#L61-L64)). A native button's platform
foreground is therefore allowed to win. In the macOS webview shown in the
screen capture, that foreground is white, on the same light page surface. The
heading is present; it is simply not legible.

This is a presentation defect only. Product selection, model classification,
and the three-card ranking path have already completed before this element is
rendered.

## Feature inputs

- **Outcome:** The fourth card is a visibly intentional alternative path from
  the three ranked dresses into the existing portfolio-search flow.
- **Actor:** A shopper who does not want any of the first three ranked dresses.
- **Boundary:** This change affects only the fourth action card on the
  discovery screen. It does not change catalog selection, propensity ranking,
  GPT category classification, or cart behaviour.
- **Failure being prevented:** Platform button foreground defaults make the
  action title indistinguishable from the light discovery canvas.
- **Runtime:** Tauri's macOS webview; styles must also remain readable in the
  browser test runtime and at the existing responsive breakpoints.

## Executable requirements

### REQ-IMPROVE-001.0: Render alternate discovery card visibly

**WHEN** the discovery screen renders three ranked product cards
**THEN** the system **SHALL** render one fourth `button.fourth-card` after them
**AND** its title, supporting text, and arrow **SHALL** be visibly distinct
from the light page canvas
**AND** the title foreground **SHALL** be explicitly specified by the
application stylesheet rather than inherited from the platform button style.

**Verification:** Inspect the generated discovery markup and computed styles
for `#something-else`.

### REQ-IMPROVE-002.0: Preserve readable contrast across button states

**WHEN** the fourth card is in default, keyboard-focus, hover, or active state
**THEN** its title and supporting copy **SHALL** retain at least a 4.5:1 text
contrast ratio against the card background
**AND** the focus indicator **SHALL** remain visible without relying only on
the dashed border.

**Verification:** A browser style test asserts explicit foreground and focus
rules; a visual E2E capture confirms all four states on the macOS webview.

### REQ-IMPROVE-003.0: Keep alternate path semantically distinct

**WHEN** the shopper sees the initial three recommendations
**THEN** the fourth card **SHALL** remain visually separate from product cards
through its dashed boundary and action-oriented copy
**AND** clicking it **SHALL** retain the existing transition to the portfolio
search form.

**Verification:** An interaction test clicks `#something-else`, asserts the
search form is shown, and asserts the selected-product state remains empty.

### REQ-IMPROVE-004.0: Keep search continuation readable

**WHEN** a portfolio search has any unseen eligible dress remaining
**THEN** the same fourth-card treatment for `#next-three` **SHALL** use the
same explicit readable foreground and interaction states with the copy
`Show 3 more`
**AND** the terminal fourth card after the final one- or two-dress page
**SHALL** use an explicit dark foreground for `No more inventory.`.

**Verification:** Render both a paginated search result and a final partial
page; assert readable fourth-card text in both states and that only the
non-final page exposes the next-page interaction.

## Recommended implementation

Keep the transparent, dashed-card visual language, but set a deliberate dark
foreground on `.fourth-card` (the existing `#261a31` application ink is a
compatible choice). Add explicit `:hover`, `:focus-visible`, and `:active`
rules; do not depend on macOS's default button colour or focus treatment.

The product-card rendering code does not need a data or control-flow change.

## Test matrix

| Test ID | Level | Scenario | Expected result |
| --- | --- | --- | --- |
| TEST-CTA-001 | Frontend render | Initial first look | `#something-else` is the fourth grid element and its three text elements are present. |
| TEST-CTA-002 | Styles | Default and focus | The fourth-card foreground and focus indicator are explicitly defined; readable text meets 4.5:1 against its background. |
| TEST-CTA-003 | Interaction | Something else | Click routes to the existing portfolio-search form without selecting or adding a product. |
| TEST-CTA-004 | Visual E2E | macOS Tauri webview | A screenshot at desktop width shows title, helper text, arrow, boundary, and focus state clearly. |
| TEST-CTA-005 | Responsive visual | 980 px and 600 px breakpoints | The fourth card remains readable when the grid becomes two and then one column. |
| TEST-CTA-006 | Search pagination | Show 3 more | `#next-three` uses the same readable action-card style while unseen inventory remains; the final page shows a readable `No more inventory.` terminal card. |

## TDD delivery sequence

1. **STUB:** Add the frontend render and interaction tests for `#something-else` and `#next-three`.
2. **RED:** Confirm the style assertion fails because `.fourth-card` has no explicit foreground or focus rule.
3. **GREEN:** Add the minimal scoped card foreground and interactive-state rules in `src/styles.css`.
4. **REFACTOR:** Extract semantic colour tokens only if doing so removes duplication without changing the already-approved product-card styling.
5. **VERIFY:** Run frontend tests, the Rust suite, production build, and inspect Tauri screenshots at all listed breakpoints.

## Quality gates

- `pnpm test` passes.
- `pnpm build` passes.
- Existing `cargo test --manifest-path src-tauri/Cargo.toml --all-targets --all-features` passes.
- The E2E capture contains no white or nearly-white fourth-card heading on the light discovery canvas.
- The card's click targets and existing IDs remain unchanged.

## Non-goals

- Changing the three initial recommendation cards or their propensity scores.
- Changing the GPT-4o category contract or catalog data.
- Adding a fourth product recommendation.
- Replacing portfolio search, product chat, or cart flows.
