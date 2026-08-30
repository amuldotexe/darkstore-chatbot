# E2E Test 001

Visual evidence for the complete v001 shopper journey.

| Transition | Evidence |
| --- | --- |
| Mounted DMG starts at the key gate | `01-installed-dmg-key-gate.png` |
| Fixture key gate | `02-key-gate-fixture.png` |
| First three propensity-ranked dresses | `03-first-look.png` |
| Select a dress | `04-product-chat.png` |
| Open the product-details hyperlink | `04b-product-details.png` |
| Ask the customization chat | `05-customization-chat.png` |
| Select a size and add to local cart | `06-local-cart.png` |
| Search a dress brief; retain it for pagination | `07-portfolio-search.png` |
| Request the next three cards | `08b-next-three.png` |
| Ask for an absent category; do not substitute | `08-absence-recovery.png` |

The browser-fixture images are cropped to the app canvas, then visually checked for clipping and overlap. `e2e-journey.ts` is an explicit non-production bridge using the same embedded data contract; it does not ship inside the DMG. The release DMG still requires a shopper-entered OpenAI key before it calls GPT-4o. No live key is stored, sent, or used to create these screenshots.
