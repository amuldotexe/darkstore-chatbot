# Commerce Chatbot UI Precedents — v01

**Research date:** 30 August 2026
**Decision supported:** Design the desktop mock's UI before selecting a production backend.
**Related artefacts:** [D01 — Problem Statement](D01-problem-statement.md), [D05 — Live Slikk Dresses Sample](D05-slikk-dresses-sample-v01.csv)

## Premise Check

**Yes—close examples already exist.** The strongest open-source precedents combine natural-language search, structured product results, and direct commerce actions rather than returning a text-only answer:

- [Turbo Start Aisle](https://github.com/robotostudio/turbo-start-aisle) is a Shopify starter with a floating chat widget, page-context capture, AI-controlled filters, inline product cards, and an add-to-cart bridge.
- [Shopping Assistant with Channel3](https://github.com/channel3-ai/shopping-assistant) searches a live product catalogue from chat and renders an interactive product carousel and product details.
- [AI Shopping Assistant](https://github.com/rock420/AI-Shopping-Assistant) explicitly streams a UI event after product-search tools return data; React then renders product cards with images, prices, attributes, and add-to-cart controls.
- [Smart E-Commerce](https://github.com/ushanchamod/smart-e-commerce) describes a React/Node demo where a chat assistant pushes interactive cards into the interface, alongside browsing, cart, and order flows.

**Important gap:** none of the reviewed examples is evidence of the exact target: a fashion-specific, dark-store experience where every card-visible stock, price, size, and delivery statement is tied to a location-scoped inventory snapshot. That reliability constraint remains a design requirement for this mock, not a solved feature to copy.

## Expert Lenses

| Lens | What it optimises for | Implication |
| --- | --- | --- |
| Fashion shopper | Confidence, fast visual comparison, low typing burden | Start from three image-forward looks; keep refinement optional and lightweight. |
| Commerce operator | Truthful availability, conversion, policy clarity | Cards must identify the selected SKU/size and show verified delivery and fulfilment signals. |
| Product engineer | Component boundaries and predictable state | Stream prose separately from a typed `product_results` UI event; the UI renders tool data, not model-invented fields. |
| Desktop interaction designer | Satisfaction at wide viewports | Use a persistent, calm evidence rail rather than a floating chat widget that competes with cards. |
| Skeptical reviewer | Hallucination, false urgency, and copied demo complexity | Do not copy agentic checkout, arbitrary tool autonomy, or client-side API-key patterns. |

## Candidate Approaches

| Approach | Strength | Failure mode | Verdict |
| --- | --- | --- | --- |
| Catalog with a floating chatbot | Familiar and easy to retrofit | Chat becomes an afterthought; results fragment across a small overlay | Too weak for the mock. |
| Full-screen text chatbot with cards in the transcript | Clear conversational flow | Images and comparison become vertically exhausting on desktop | Useful for a support bot, not fashion discovery. |
| Traditional fashion grid plus filters | Proven browsing model | Does not demonstrate why a conversational layer matters | Necessary fallback, insufficient hero experience. |
| **Concierge workbench: chat + three-look rack + evidence rail** | Makes the recommendation, the merchandise, and the proof visible together | Requires disciplined layout and typed UI events | **Choose this hybrid.** |

### Non-obvious but useful blends

1. **Fitting-room pattern:** Treat the first three cards as a small, editorial fitting rail—not an endless search result. The shopper can compare, save, or replace one look without restarting.
2. **Air-traffic-control pattern:** Borrow Confido's visible status and activity language for system truth, but make it shopper-readable: `Inventory checked 10:42`, `Size M available`, `Delivery by 6:30 pm`.
3. **Progressive-disclosure pattern:** Start with visual cards and one sentence of rationale. Reveal specifications, alternatives, and exclusion reasons only after a click or follow-up.

## Chosen Thesis

Design a **desktop fashion concierge**:

- **Main canvas:** a conversation that opens with three large, shoppable look cards—not a blank prompt.
- **Card contract:** image, title, price, available sizes, delivery promise, fulfilment flag (for example, Try & Buy), one concise `Why this` explanation, and a direct action.
- **Conversation contract:** chips for common pivots (`Cheaper`, `More formal`, `Different colour`, `Something else`) plus natural-language input.
- **Evidence rail:** a narrow, persistent right-hand column inspired by Confido's activity log. It shows source-of-truth status and a compact decision trace, not internal model reasoning.

This is a stronger blend than a direct Slikk clone: Slikk supplies the fast-commerce merchandising signals; Confido supplies calm hierarchy, explicit readiness, progress states, and confidence-building evidence.

## What to Borrow—and What Not to Borrow

### From Slikk

The [Slikk storefront](https://www.slikk.club/) demonstrates rapid-delivery merchandising: a location-contextual delivery promise, highly visual category navigation, price/MRP/discount display, and visible trust/policy markers. Its product pages also surface variants and structured fashion attributes. Use the *merchandising grammar*, not its category-first home-page layout.

Borrow:

- image-forward product cards;
- price, MRP, discount, size, and fulfilment information at a glance;
- direct `Add to bag` and product-detail actions;
- location-aware delivery as a first-class signal.

Do not borrow:

- the broad, endless home-page rail structure;
- price-led badges as the sole explanation for recommendation quality;
- an experience that makes the shopper hunt before seeing relevant looks.

### From Confido Exploration

The local [Confido Exploration repository](https://github.com/amuldotexe/confido-exploration-01) has a Tauri/Vite workbench with a main surface plus persistent activity rail, warm neutral canvas, restrained teal accent, status chips, progress states, evidence tables, and clear enabled/disabled actions.

Borrow:

- the wide desktop shell: productive main canvas plus a stable right rail;
- warm paper-like base, dark ink, one restrained accent colour, clear panel boundaries;
- explicit progress and recoverable error states;
- a visible, human-readable activity trace that explains system state.

Do not borrow:

- the API-key entry panel in the shopper flow;
- dense operator tables as the primary shopper surface;
- the prompt-review vocabulary. The rail must talk about inventory and delivery, not model operations.

## Codebase Shortlist

| Reference | Relevance | Reuse / inspect | Caveat |
| --- | --- | --- | --- |
| [robotostudio/turbo-start-aisle](https://github.com/robotostudio/turbo-start-aisle) | Closest complete commerce-chat architecture | Chat widget, page-context capture, filter control, inline cards, cart bridge | It is Shopify/Sanity and uses a different provider configuration. |
| [channel3-ai/shopping-assistant](https://github.com/channel3-ai/shopping-assistant) | Closest chat-result visual pattern | Streaming chat, interactive product carousel, detail action | Depends on the Channel3 catalogue/API. |
| [rock420/AI-Shopping-Assistant](https://github.com/rock420/AI-Shopping-Assistant) | Best explicit truth-boundary architecture | Tool result → typed UI event → React product list | Framework and README claims need independent implementation review. |
| [ushanchamod/smart-e-commerce](https://github.com/ushanchamod/smart-e-commerce) | Full conversational commerce workflow | Interactive cards, action suggestions, RAG/search split | More agent complexity than the mock needs. |
| [vercel/chatbot](https://github.com/vercel/chatbot) | Strong implementation scaffold | Streaming, message lifecycle, production chat conventions | Not commerce-specific. |
| [assistant-ui/assistant-ui](https://github.com/assistant-ui/assistant-ui) | Best composable React chat primitives | Thread, composer, action bar, typed tool/JSON rendering | Adds a UI library; evaluate against a purpose-built Tauri view. |
| [openai/chatkit-js](https://github.com/openai/chatkit-js) | Fastest rich-chat proof of concept | Streaming, widgets, threads, source annotations | Requires evaluating its managed/self-hosted trade-offs for a Tauri desktop app. |
| [channel3-ai/channel3-ui](https://github.com/channel3-ai/channel3-ui) | High-quality commerce component reference | Product card, carousel, variant selector, offer list, attributes | Its API types are specific to Channel3; treat as a design/component reference. |

## Recommended UI Contract for the Mock

### First screen

1. Header: delivery location and verified delivery window.
2. Concierge greeting: one sentence explaining why these three looks were selected.
3. Three-card horizontal rack with image, selected variant, price, size state, delivery, `Why this`, and `View` / `Add to bag` actions.
4. Refinement chips beneath the rack; a prominent `Looking for something else?` entry point.
5. Right evidence rail: `Inventory checked`, `0 unsupported claims`, delivery eligibility, and an expandable `Why not these?` list.

### Typed events—not model-shaped cards

The service should emit a small, validated contract such as:

```text
assistant_text
product_results { inventory_snapshot_at, location_id, items[] }
clarification { question, options[] }
recommendation_trace { filters_applied, excluded_count }
```

`items[]` must be produced by the retail recommendation core after it verifies inventory, price, variant, and delivery—not composed freely by the LLM. This matches the key pattern documented in [AI Shopping Assistant](https://github.com/rock420/AI-Shopping-Assistant): React receives structured UI context derived from tool data and renders product cards from that data.

## Evidence and Verification

| Verification question | Evidence | Result |
| --- | --- | --- |
| Does an open-source project already show chat product results as UI, not text? | Channel3 renders an interactive product carousel; Rock420 and Smart E-Commerce describe interactive product cards. | Yes. |
| Does an open-source implementation connect a chat card to a commerce action? | Turbo Start Aisle documents an add-to-cart bridge from chat product cards to a Shopify cart. | Yes. |
| Is an open-source chat UI available without building every message primitive? | assistant-ui provides composable chat primitives and tool/JSON rendering; OpenAI ChatKit provides a complete UI with rich widgets. | Yes. |
| Can a model key be safely placed in the desktop/browser UI? | OpenAI directs that API keys stay server-side and out of client code; ChatKit also mints short-lived client credentials from a server. | No. |
| Does the research prove the exact dark-store reliability experience exists? | The reviewed repositories establish interaction and architecture patterns, but not location-scoped fashion inventory guarantees. | No—this is the differentiation. |

## API-Key Handling

The project needs **one local development secret**, not a copied or discoverable secret: `OPENAI_API_KEY`. It belongs in an ignored local environment file and must be read by the backend only. [OpenAI's API-key guidance](https://help.openai.com/en/articles/5112595-best-practices-for-api-key-safet) says not to expose keys in browsers or mobile apps and not to commit them. Use a project-scoped development key with a spend limit; do not reuse or share a personal key.

For this repository, the intended local file is:

```text
git-references/darkstore-chatbot.gpt4o.local.env
```

It is intentionally ignored. It contains an empty `OPENAI_API_KEY` placeholder only; a developer must add their own project key locally.

## Final Synthesis

Build the UI as a **verified fashion concierge**, not a generic chat window. Start from the three visual looks, make card actions immediate, and use a Confido-inspired side rail to make inventory, delivery, and decision state legible. Take interaction patterns from Turbo Start Aisle, Channel3 Shopping Assistant, and the tool-data-to-UI separation from AI Shopping Assistant; use assistant-ui or OpenAI ChatKit only if their integration costs are lower than a focused Tauri implementation.

## Open Questions

1. Should the first UI prototype use a custom Tauri/Vite view, `assistant-ui`, or OpenAI ChatKit?
2. Which card action is in scope for the mock: open a PDP, add a mock bag item, or both?
3. What is the mock's delivery-policy language when an eligible size is absent?
4. Should the right evidence rail be visible by default, or collapse after first use?
