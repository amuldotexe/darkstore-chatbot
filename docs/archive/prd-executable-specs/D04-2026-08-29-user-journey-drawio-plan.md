# User Journey Draw.io Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use `executing-plans` to implement this plan task-by-task.

**Goal:** Create an editable diagrams.net journey showing the dark-store fashion discovery, refinement, and recovery flows for product engineers.

**Status:** Superseded. Its completed artifact is retained at `docs/diagrams/archive/user-journeys/darkstore-fashion-first-look-v01.drawio`; the v001 PRD source of truth is `docs/diagrams/darkstore-propensity-cart-journey-v001.drawio`.

**Architecture:** One diagrams.net page uses swimlanes for Shopper, Tauri desktop UI, AI intent service, and Retail recommendation core. Numbered action nodes establish the walkthrough order; decision nodes and tinted notes make the availability/trust boundary and recovery path explicit.

**Tech Stack:** diagrams.net XML (`mxfile` / `mxGraphModel`), XML validation with `xmllint`, semantic validation with Python standard-library XML parsing.

---

### Task 1: Create the editable swimlane diagram

**Files:**
- Archived source: `docs/diagrams/archive/user-journeys/darkstore-fashion-first-look-v01.drawio`
- Archived reference: `docs/archive/prd-executable-specs/D01-problem-statement.md`
- Reference: `docs/D02-conversational-fashion-precedents.md`
- Archived reference: `docs/archive/prd-executable-specs/D03-2026-08-29-user-journey-design.md`

**Step 1: Verify the archived historical artifact**

Run:

```bash
test -e docs/diagrams/archive/user-journeys/darkstore-fashion-first-look-v01.drawio
```

Expected: exit code `0`; the historical artifact is retained only for reference.

**Step 2: Add the diagrams.net XML artifact**

Create a single `mxfile` page containing:

- four labelled horizontal swimlanes: Shopper, Tauri Desktop UI, AI Intent Service, Retail Recommendation Core;
- numbered nodes for opening the app, preparing initial looks, displaying cards, browsing/add-to-bag, free-text refinement, structured intent parsing, deterministic validation/ranking, and the refreshed selection;
- a decision diamond for whether a clarification materially changes the viable set;
- an unavailable-inventory recovery node with four actionable alternatives;
- a red-tinted note stating that the LLM cannot assert stock, price, size, or delivery facts;
- a yellow-tinted note limiting clarification to one high-information question;
- dashed trace edges to a decision/event trace node.

Use standard `mxGraphModel` geometry, `swimlane` styles, and individually editable vertices/edges. Do not embed the diagram as an image or encode it as a compressed blob.

**Step 3: Validate well-formed XML**

Run:

```bash
xmllint --noout docs/diagrams/archive/user-journeys/darkstore-fashion-first-look-v01.drawio
```

Expected: exit code `0` and no output.

### Task 2: Validate the implementation contract is present in the diagram

**Files:**
- Archived test target: `docs/diagrams/archive/user-journeys/darkstore-fashion-first-look-v01.drawio`

**Step 1: Run a semantic-label check**

Run:

```bash
python3 - <<'PY'
from pathlib import Path
from xml.etree import ElementTree

diagram_path = Path("docs/diagrams/archive/user-journeys/darkstore-fashion-first-look-v01.drawio")
root = ElementTree.parse(diagram_path).getroot()
labels = "\n".join(
    cell.attrib.get("value", "")
    for cell in root.iter("mxCell")
)
required_labels = (
    "Shopper",
    "Tauri Desktop UI",
    "AI Intent Service",
    "Retail Recommendation Core",
    "Show three shoppable looks",
    "Looking for something else",
    "Structured intent only",
    "Validate inventory, price, size, and delivery",
    "Ask one high-information clarification",
    "No eligible look",
    "LLM never asserts stock, price, size, or delivery facts",
)
missing_labels = [label for label in required_labels if label not in labels]
if missing_labels:
    raise SystemExit(f"Missing labels: {missing_labels}")
print("Diagram labels cover the approved journey contract.")
PY
```

Expected: `Diagram labels cover the approved journey contract.`

**Step 2: Visually inspect editability in diagrams.net**

Open `docs/diagrams/archive/user-journeys/darkstore-fashion-first-look-v01.drawio` in diagrams.net and confirm that the swimlanes, nodes, and edges are independently selectable and that the recovery branch is visible without scrolling horizontally past the canvas bounds. This historical plan is superseded by `docs/diagrams/darkstore-propensity-cart-journey-v001.drawio`.

**Step 3: Check repository state**

Run:

```bash
git diff --check
git status --short
```

Expected: no whitespace errors; report the newly created diagram and supporting design/plan notes. Do not commit unless the user explicitly requests a commit.
