# User Journey Diagram Design

**Date:** 29 August 2026
**Audience:** Product engineering team
**Output:** A diagrams.net (`.drawio`) implementation journey diagram

## Objective

Make the intended dark-store fashion experience implementation-ready without turning the diagram into a complete technical architecture. The diagram must show who performs each step, where the AI boundary is, and how the system prevents inaccurate commerce claims.

## Selected Diagram Pattern

Use a left-to-right swimlane journey with four lanes:

1. **Shopper** — actions and decisions visible to the customer.
2. **Tauri desktop UI** — visual cards, controls, loading states, and screen transitions.
3. **AI intent service** — constrained conversion of natural language to an intent/clarification response.
4. **Retail recommendation core** — deterministic inventory eligibility, ranking, and event tracing.

The swimlane approach was selected over a linear journey because it assigns engineering ownership and shows asynchronous boundaries. It was selected over a state machine because it remains readable as the shared first implementation artefact.

## Journey Scope

### Entry journey

1. Shopper opens the desktop application.
2. Retail core reads the current fixture inventory snapshot and shopper context.
3. Retail core filters out items that are unavailable, lack an eligible size, miss the delivery promise, or fail the budget constraint when one is known.
4. UI displays three shoppable look cards with product facts and visible reason codes.
5. Shopper can inspect a look, choose a size, add it to the bag, or ask for something else.

### Refinement journey

1. Shopper enters a free-text need.
2. AI service returns a structured intent or a single proposed clarification; it does not return product facts.
3. Retail core validates the intent against the inventory snapshot and ranks eligible looks.
4. If a missing fact materially changes the viable shortlist, UI asks one clarification question; otherwise it immediately displays a refreshed set of three cards.
5. Shopper can refine with a chip, inspect a look, choose a size, or add it to the bag.

### Failure/recovery journey

1. If no eligible look exists, UI explains the unmet constraint without implying that unavailable stock can be purchased.
2. UI offers actionable recovery: relax budget, select another colour/category, remove a size constraint, or see the closest deliverable alternative.
3. Retail core records the exclusion reasons and interaction outcome.

## Visual Conventions

- Rounded rectangles: action or processing step.
- Diamonds: decision gate.
- Red-tinted note: non-negotiable commerce truth boundary.
- Yellow-tinted note: product/questioning heuristic.
- Solid arrows: synchronous journey progression.
- Dashed arrows: data and trace writes.
- Step labels (`1`–`8`): reading order for a walkthrough.

## Acceptance Criteria

- The file opens in diagrams.net without missing XML structure.
- A reader can explain the shopper happy path in under two minutes.
- The diagram distinguishes LLM intent extraction from deterministic retail facts.
- The diagram contains an unavailable-inventory recovery path.
- Every card-visible product fact has a clear owner in the retail core.
- The journey ends in a direct add-to-bag action or a transparent recovery, not an open-ended chat state.

## Out of Scope

- Real-time external inventory integrations.
- Payment, checkout, warehouse picking, and last-mile delivery workflows.
- The exact ranking formula, prompt wording, and database schema.
- A production privacy, consent, or observability architecture.
