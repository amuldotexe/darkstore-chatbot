# Turso Dress Data V001 Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan task-by-task.

**Goal:** Supply the v001 concierge with a tiny, reproducible Turso/libSQL catalogue: eight Slikk-derived dresses, one supported category, and two or three selectable sizes per product.

**Architecture:** Keep the existing CSV as captured-source evidence. Create one Turso-compatible SQL dump that creates a single strict `inventory_products` table and inserts eight product rows. The application later reads a distinct category list from that table; it does not maintain a second taxonomy table. GPT may select only `dresses` or return `not_in_inventory`, while local SQL ordering supplies the top three by propensity.

**Tech Stack:** Turso Cloud/libSQL, SQLite-compatible SQL, `sqlite3` only for local compatibility verification, Tauri/Rust design contracts.

---

### Task 1: Define bounded catalogue contract

**Files:**

- Create: `docs/plans/D09-turso-dress-data-v001.md`
- Modify: `docs/specs-v001.md`
- Modify: `docs/journals/turso-dress-data-v001.md`

**Step 1: State the scope**

Define one supported category ID, `dresses`, and exactly eight products sourced from `docs/D05-slikk-dresses-sample-v01.csv`.

**Step 2: Separate fact provenance**

Retain source-derived SKU, brand, title, price, discount, Try & Buy flag, source URL, and capture time. Explicitly label availability, sizes, delivery window, tags, and propensity score as v001 demo-fixture values.

**Step 3: Define unsupported-category response**

Require a `not_in_inventory` result with no product cards when the shopper asks for a category other than dresses. The response names dresses as the current scope and leaves the brief editable.

### Task 2: Create Turso-compatible seed dump

**Files:**

- Create: `data/darkstore-dresses-v001.sql`
- Preserve: `docs/D05-slikk-dresses-sample-v01.csv`

**Step 1: Write the failing compatibility query**

Run the future dump against an in-memory SQLite database and query for one table, eight rows, one distinct `category_id`, and three top-ranked products. It must fail before the dump exists.

**Step 2: Write the minimum schema**

Create exactly one `STRICT` table, `inventory_products`, with source provenance plus the required demo-fixture columns. Constrain `category_id` to `dresses`, availability to boolean values, sizes to two or three values encoded as JSON text, and propensity scores to an integer from 0 through 100.

**Step 3: Seed the eight rows**

Use the captured eight Slikk records. Assign deterministic demo propensity scores so the initial three are stable, and add two or three available size options to every record.

**Step 4: Verify compatibility and data invariants**

Run `sqlite3 :memory: < data/darkstore-dresses-v001.sql`, then assert: one table, eight rows, all products in `dresses`, every row is demo-available, every size list has two or three values, and the first three descending scores are deterministic.

### Task 3: Connect Turso deployment contract

**Files:**

- Create: `.env.example`
- Modify: `docs/specs-v001.md`

**Step 1: Declare connection variables**

Add blank `TURSO_DATABASE_URL` and `TURSO_AUTH_TOKEN` examples only. Do not add actual credentials to the repository.

**Step 2: Specify deployment boundary**

Document the user-operated Turso creation path: create `darkstore-dresses-v001` from the SQL dump, then provide the database URL and token only through local environment variables.

**Step 3: Specify Rust boundary**

Require a Rust-side libSQL client to query the database; the WebView receives catalog DTOs only and no Turso credentials.

### Task 4: Update product and recovery contracts

**Files:**

- Modify: `docs/specs-v001.md`

**Step 1: Update category selection contract**

Use a tagged model result: `matched` with `dresses`, or `not_in_inventory` with a shopper-safe acknowledgement. Reject product IDs and all unsupported category IDs.

**Step 2: Add absent-category requirement**

Add a traceable requirement and test row that a shirts, shoes, or jacket request shows no product cards and offers the Dresses-only scope.

**Step 3: Update ranking contract**

Specify a local SQL query ordered by `propensity_score DESC, sku ASC`, with only available dress rows eligible.

### Task 5: Update visual source of truth

**Files:**

- Modify: `docs/diagrams/darkstore-propensity-cart-journey-v001.drawio`
- Modify: `docs/diagrams/darkstore-propensity-cart-journey-v001.png`

**Step 1: Show exact runtime inventory**

Replace the generic fixture node with the one-table, eight-dress Turso catalogue.

**Step 2: Show absent-category recovery**

Add a visible route from a free-text brief to “not in inventory: currently dresses only”, with no cards rendered and a brief-revision action.

**Step 3: Export and inspect**

Export the Drawio source at full page size. Inspect the PNG for clipping, overlapping paths, and readable recovery wording.

### Task 6: Final verification

**Files:**

- Verify: `data/darkstore-dresses-v001.sql`
- Verify: `docs/specs-v001.md`
- Verify: `docs/diagrams/darkstore-propensity-cart-journey-v001.drawio`
- Verify: `docs/diagrams/darkstore-propensity-cart-journey-v001.png`

**Step 1: Validate SQL invariants**

Run the full in-memory database check from Task 2.

**Step 2: Validate document traceability**

Ensure every `REQ-TAURI-*` has one verification-matrix row and the specification never exposes a Turso token to the frontend.

**Step 3: Validate Drawio and export**

Validate XML, check the PNG dimensions, inspect its rendered visual, and run `git diff --check`.
