# D07 — Commerce Chatbot Codebase Comparison v01

**Purpose:** Compare the local reference checkouts for the dark-store fashion
chatbot before we choose a UI implementation path.

**Snapshot:** 2026-08-30. The eight public repositories below are local,
ignored, shallow (`--depth 1 --single-branch`) clones in `git-references/`.
They are for reading and pattern extraction—not source code to merge blindly.
`confido-exploration-01` was already present as a normal local clone, so it is
intentionally called out as **not shallow**.

## Executive recommendation

Do not fork a full reference application. Build the first desktop prototype as
a **Confido-style Tauri workbench** with a purpose-built fashion conversation
surface:

1. A visual opening with three shoppable looks, grounded in locally available
   inventory.
2. A conversational refinement panel where product results are rendered as
   typed UI events, not prose that happens to contain product names.
3. A narrow evidence/activity rail that explains the current session state:
   availability, delivery promise, chosen filters, and why a look was shown.

Borrow the interaction contract from Turbo Start Aisle and Channel3 Shopping
Assistant: **chat/tool result → product cards → an explicit customer action**.
Borrow the desktop composition and honest system feedback from Confido. Keep
the backend intentionally narrow for the prototype; full checkout,
multi-agent routing, embeddings, and external-catalog integrations are later
concerns.

## Checkout inventory

| Reference | Local checkout | Revision | Clone depth | Primary role |
| --- | --- | --- | --- | --- |
| [Confido](https://github.com/amuldotexe/confido-exploration-01) | `git-references/confido-exploration-01` | `0aab0a2` | Existing normal clone | Desktop workbench language |
| [Turbo Start Aisle](https://github.com/robotostudio/turbo-start-aisle) | `git-references/turbo-start-aisle` | `79170f5` | shallow | Closest commerce-chat interaction |
| [Channel3 Shopping Assistant](https://github.com/channel3-ai/shopping-assistant) | `git-references/channel3-shopping-assistant` | `7fc3f5e` | shallow | Search-result carousel in chat |
| [AI Shopping Assistant](https://github.com/rock420/AI-Shopping-Assistant) | `git-references/ai-shopping-assistant-rock420` | `f1d82de` | shallow | Conversational checkout architecture |
| [Smart E-Commerce](https://github.com/ushanchamod/smart-e-commerce) | `git-references/smart-e-commerce` | `0d10d2e` | shallow | End-to-end assistant/backend demo |
| [Vercel Chatbot](https://github.com/vercel/chatbot) | `git-references/vercel-chatbot` | `c2f8235` | shallow | Polished general chat baseline |
| [assistant-ui](https://github.com/assistant-ui/assistant-ui) | `git-references/assistant-ui` | `5bdd416` | shallow | Composable React chat primitives |
| [OpenAI ChatKit JS](https://github.com/openai/chatkit-js) | `git-references/openai-chatkit-js` | `2261384` | shallow | Packaged, rich chat surface |
| [Channel3 UI](https://github.com/channel3-ai/channel3-ui) | `git-references/channel3-ui` | `c46e99f` | shallow | Product-search and PDP components |

## Side-by-side evaluation

| Reference | What it demonstrably contributes | Stack / integration cost | Dark-store fit | Take, adapt, avoid |
| --- | --- | --- | --- | --- |
| **Confido** | Tauri desktop shell; calm warm-neutral panels; explicit analysis status; a useful 320px activity/evidence rail. | Already aligned with the proposed desktop direction; it is not a shopper chat or commerce engine. | **High for shell, low for commerce.** | Take layout rhythm, progressive status, and evidence rail. Adapt its professional tone into fashion language. Do not copy its analyst workflow. |
| **Turbo Start Aisle** | Floating chat, page-context capture, AI-controlled filters, inline product cards, and an explicit card-to-cart bridge. | Turborepo + Next + Shopify + Sanity + Vercel AI Gateway/MCP: substantial infrastructure. | **Highest interaction fit.** | Take the typed flow from agent result to card to cart intent. Do not import its Shopify/Sanity/Vercel stack into the first Tauri prototype. |
| **Channel3 Shopping Assistant** | Natural-language catalog search rendered as an interactive carousel, with drill-in product details. | Next.js + React + Vercel AI SDK + Channel3 catalog. Vendor/data-provider coupling. | **High for the result experience.** | Take carousel density, product-detail handoff, and quick refinement prompts. Replace Channel3 fetches with the dark-store inventory API. |
| **AI Shopping Assistant** | A thorough conversational checkout model: discovery, basket actions, price snapshots, inventory reservation, and structured confirmation for sensitive actions. | A full application/domain design with multi-agent routing and OpenAI function calling. | **Medium now, high later.** | Treat it as a checkout and inventory-consistency reference. Avoid multi-agent complexity and payment flows during a UI-first prototype. |
| **Smart E-Commerce** | Interactive product cards emitted into chat, order/policy responses, and an end-to-end commerce workflow. | Vite/React client plus Node/Express, PostgreSQL + pgvector, LangChain/LangGraph, WebSockets, and auth. | **Medium.** | Use it to pressure-test later backend boundaries. Avoid adopting its RAG, agent graph, persistence, and operations footprint for the first screen. |
| **Vercel Chatbot** | A polished, extensible chat baseline: streaming, tools/generative UI, persistence, authentication, attachments, and tests. | Next.js App Router, AI SDK, Postgres, Blob, Auth.js, Vercel-specific integrations. | **Medium for chat ergonomics; low for commerce semantics.** | Take patterns for reliable streaming, history, retry states, and testing. Do not let a generic chat template dictate the product browsing surface. |
| **assistant-ui** | Composable TypeScript/React primitives for threads, messages, composer, action bars, tool-call UI, approval UI, and typed runtimes. | React required; can connect to a custom backend but is a library rather than a complete desktop app. | **High if the desktop UI is React.** | Strong candidate if we choose React inside Tauri and want full visual ownership. Avoid adding it to the current vanilla-TS shell without an explicit framework decision. |
| **OpenAI ChatKit JS** | Ready-made streaming chat, rich widgets, attachments, thread management, and server-minted client sessions. | React binding or web component integration; needs a server endpoint to mint client tokens. | **High for fast conversational polish, medium for bespoke commerce UI.** | Evaluate for prototype speed if its widget model supports the exact three-look/card UX. Keep token minting server-side; never embed an API key in the desktop client. |
| **Channel3 UI** | Well-specified product search, filters, grid, product card, PDP, variants, offers, attributes, and recommendations. Components are presentational and callback-driven. | React + Tailwind/shadcn; types are directly coupled to Channel3's SDK data shape. | **High for visual commerce patterns; low for direct reuse.** | Take information hierarchy and callback-driven components. Adapt the schemas to the local dark-store catalog; do not adopt a third-party catalog as the source of truth. |

## What this means for the first desktop screen

### The desired composition

| Region | Job | Reference influence |
| --- | --- | --- |
| **Look canvas** | Present exactly three look cards immediately, each showing product image, price, available size, delivery promise, and a concise reason. | Slikk product-card grammar plus Channel3 carousel discipline |
| **Conversation pane** | Let the shopper ask for a different occasion, fit, colour, price point, or item; replies render cards/filters as structured components. | Turbo Start Aisle and Channel3 Shopping Assistant |
| **Evidence rail** | Make freshness visible: inventory checked, size/colour selection, ETA, budget and trend signals. It should support—not distract from—the shopper. | Confido |
| **Product drawer / detail** | Expand a chosen card in context; offer a clear page/deep-link action instead of burying the shopper in a traditional storefront. | Channel3 Shopping Assistant and Channel3 UI |

### The minimal typed contract

The model should never invent a SKU, price, availability, or promise. It may
request a backend tool; the client renders the returned truth.

```ts
type ProductCard = {
  sku: string;
  title: string;
  imageUrl: string;
  priceInr: number;
  availableSizes: string[];
  deliveryPromise: string;
  rationale: string;
};

type ChatUiEvent =
  | { type: "show_looks"; heading: string; looks: ProductCard[] }
  | { type: "show_products"; products: ProductCard[] }
  | { type: "show_refinement"; options: string[] }
  | { type: "show_evidence"; facts: string[] };
```

This lets the backend own inventory truth and lets the UI own the shopper
experience. It is the common seam across the strongest references, without
copying their systems.

## Decision gates

1. **Keep the current Tauri + vanilla TypeScript UI:** build small custom
   chat/card components. This is the lowest-risk path for a visually specific
   prototype and keeps Confido's strongest qualities.
2. **Adopt React inside Tauri:** choose `assistant-ui` when component-level
   control, custom cards, and a custom backend are more important than speed
   of initial setup.
3. **Adopt ChatKit:** choose it only after confirming that its rich widgets can
   faithfully render the three-look opening and product actions. Its server
   session endpoint is mandatory and API credentials remain backend-only.

No path should begin by importing a complete storefront. The first version
needs to validate whether people prefer the three looks, understand *why* they
were picked, and can redirect the conversation with little effort.

## Recommended reference-reading order

1. **Confido:** shell, visual system, activity/evidence rail.
2. **Turbo Start Aisle:** chat tool definitions, product-card event, cart
   bridge, page-context tracking.
3. **Channel3 Shopping Assistant:** result carousel and product-detail
   behavior.
4. **assistant-ui and ChatKit:** framework decision only after the interaction
   spec is stable.
5. **AI Shopping Assistant, Smart E-Commerce, Vercel Chatbot, and Channel3
   UI:** later references for checkout consistency, production operations, and
   detailed commerce components.

## Scope boundary

The repositories are ignored through `/git-references/` and intentionally
excluded from version control. No credentials were copied into them. The
project's local GPT configuration remains an empty, ignored template at
`git-references/darkstore-chatbot.gpt4o.local.env`; it must only be populated
locally by an authorised developer with a scoped API key.
