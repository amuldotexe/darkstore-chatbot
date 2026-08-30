import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  createConciergeApplication,
  type ConciergeBridge,
  type ProductCard,
} from "./app";

const fixtureCards: ProductCard[] = [
  {
    sku: "SKID00083927",
    category_id: "dresses",
    brand: "Slikk X Revolte",
    product_name: "Black Minimalist A-Line Evening Dress",
    current_price_inr: 1230,
    fixture_available: true,
    fixture_sizes: ["S", "M", "L"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 94,
    fixture_dress_type: "evening_dress",
    fixture_style_tags: ["minimalist", "date_night"],
    source_product_url: "https://www.slikk.club/example",
  },
  {
    sku: "SKID00174036",
    category_id: "dresses",
    brand: "MYWISHBAG",
    product_name: "Edgy Floral Party Bodycon Dress",
    current_price_inr: 1300,
    fixture_available: true,
    fixture_sizes: ["S", "M"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 92,
    fixture_dress_type: "party_dress",
    fixture_style_tags: ["party", "bodycon"],
    source_product_url: "https://www.slikk.club/example",
  },
  {
    sku: "SKID00081801",
    category_id: "dresses",
    brand: "Slikk X Revolte",
    product_name: "Black Ruched Tube Dress",
    current_price_inr: 432,
    fixture_available: true,
    fixture_sizes: ["XS", "S", "M"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 89,
    fixture_dress_type: "tube_dress",
    fixture_style_tags: ["ruched", "date_night"],
    source_product_url: "https://www.slikk.club/example",
  },
];

function createFixtureConciergeBridge(): ConciergeBridge {
  return {
    configureSessionOpenaiKey: vi.fn().mockResolvedValue({ concierge_enabled: true }),
    loadInitialProductTrio: vi.fn().mockResolvedValue({
      kind: "cards",
      category_id: "dresses",
      rationale: "A concise dress edit for tonight.",
      cards: fixtureCards,
      show_next_three: true,
    }),
    searchPortfolioProductsPage: vi.fn().mockResolvedValue({
      kind: "not_in_inventory",
      category_id: null,
      rationale: "This v001 demo currently carries dresses only.",
      cards: [],
      show_next_three: false,
    }),
    selectProductChatContext: vi.fn().mockResolvedValue({
      product: fixtureCards[0],
      selection_source: "first_look",
      retained_brief: "Help me choose a dress for tonight.",
    }),
    updateProductVariantSelection: vi.fn().mockResolvedValue(undefined),
    addValidatedVariantCart: vi.fn().mockResolvedValue({ item_count: 1, items: [] }),
  };
}

async function settleDomEvents(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

describe("TEST-FRONTEND-GUIDED-CART v001 concierge", () => {
  let root: HTMLElement;
  let bridge: ConciergeBridge;

  beforeEach(() => {
    document.body.innerHTML = '<main id="app"></main>';
    root = document.querySelector<HTMLElement>("#app")!;
    bridge = createFixtureConciergeBridge();
  });

  it("REQ-TAURI-001 through 008 moves from key gate to a size-gated local cart", async () => {
    const app = createConciergeApplication(root, bridge);
    app.mountApplicationScreen();

    const keyInput = root.querySelector<HTMLInputElement>("#api-key")!;
    keyInput.value = "sk-test-key-that-is-never-sent";
    root.querySelector<HTMLFormElement>("#key-gate")!.requestSubmit();
    await settleDomEvents();

    expect(bridge.configureSessionOpenaiKey).toHaveBeenCalledWith(
      "sk-test-key-that-is-never-sent",
    );
    expect(bridge.loadInitialProductTrio).toHaveBeenCalledOnce();
    expect(root.textContent).toContain("Black Minimalist A-Line Evening Dress");
    expect(root.textContent).toContain("Embedded catalogue facts and local propensity scores choose the products.");
    expect(root.querySelectorAll("[data-product-sku]")).toHaveLength(3);

    root.querySelector<HTMLButtonElement>('[data-select-sku="SKID00083927"]')!.click();
    await settleDomEvents();
    expect(bridge.selectProductChatContext).toHaveBeenCalledWith(
      "SKID00083927",
      "first_look",
    );

    const addButton = root.querySelector<HTMLButtonElement>("#add-cart")!;
    expect(addButton.disabled).toBe(true);
    const sizeSelect = root.querySelector<HTMLSelectElement>("#selected-size")!;
    sizeSelect.value = "M";
    sizeSelect.dispatchEvent(new Event("change", { bubbles: true }));
    await settleDomEvents();
    expect(bridge.updateProductVariantSelection).toHaveBeenCalledWith("SKID00083927", "M");

    root.querySelector<HTMLButtonElement>("#add-cart")!.click();
    await settleDomEvents();
    expect(bridge.addValidatedVariantCart).toHaveBeenCalledWith("SKID00083927", "M");
    expect(root.textContent).toContain("Cart 1");

    root.querySelector<HTMLButtonElement>("#chat-more")!.click();
    expect(root.textContent).toContain("Cart 1");
    expect(root.textContent).toContain("local v001 category list");
  });

  it("REQ-TAURI-014 shows no substitute cards for a different requested category", async () => {
    const app = createConciergeApplication(root, bridge);
    app.mountApplicationScreen();
    const keyInput = root.querySelector<HTMLInputElement>("#api-key")!;
    keyInput.value = "sk-test-key-that-is-never-sent";
    root.querySelector<HTMLFormElement>("#key-gate")!.requestSubmit();
    await settleDomEvents();

    root.querySelector<HTMLButtonElement>("#something-else")!.click();
    const briefInput = root.querySelector<HTMLInputElement>("#portfolio-brief")!;
    briefInput.value = "A linen shirt for Goa";
    root.querySelector<HTMLFormElement>("#portfolio-search")!.requestSubmit();
    await settleDomEvents();

    expect(bridge.searchPortfolioProductsPage).toHaveBeenCalledWith(
      "A linen shirt for Goa",
      false,
    );
    expect(root.textContent).toContain("currently carries dresses only");
    expect(root.querySelectorAll("[data-product-sku]")).toHaveLength(0);
  });

  it("REQ-TAURI-018 retains the search brief when requesting the next three cards", async () => {
    vi.mocked(bridge.searchPortfolioProductsPage).mockResolvedValue({
      kind: "cards",
      category_id: "dresses",
      rationale: "A complete dress edit.",
      cards: fixtureCards,
      show_next_three: true,
    });
    const app = createConciergeApplication(root, bridge);
    app.mountApplicationScreen();
    const keyInput = root.querySelector<HTMLInputElement>("#api-key")!;
    keyInput.value = "sk-test-key-that-is-never-sent";
    root.querySelector<HTMLFormElement>("#key-gate")!.requestSubmit();
    await settleDomEvents();

    root.querySelector<HTMLButtonElement>("#something-else")!.click();
    const briefInput = root.querySelector<HTMLInputElement>("#portfolio-brief")!;
    briefInput.value = "A black dress for dinner";
    root.querySelector<HTMLFormElement>("#portfolio-search")!.requestSubmit();
    await settleDomEvents();

    root.querySelector<HTMLButtonElement>("#next-three")!.click();
    await settleDomEvents();

    expect(bridge.searchPortfolioProductsPage).toHaveBeenLastCalledWith(
      "A black dress for dinner",
      true,
    );
  });

  it("REQ-TAURI-007 recovers a failed cart recheck with a fresh alternatives request", async () => {
    const app = createConciergeApplication(root, bridge);
    app.mountApplicationScreen();
    const keyInput = root.querySelector<HTMLInputElement>("#api-key")!;
    keyInput.value = "sk-test-key-that-is-never-sent";
    root.querySelector<HTMLFormElement>("#key-gate")!.requestSubmit();
    await settleDomEvents();
    root.querySelector<HTMLButtonElement>('[data-select-sku="SKID00083927"]')!.click();
    await settleDomEvents();
    const sizeSelect = root.querySelector<HTMLSelectElement>("#selected-size")!;
    sizeSelect.value = "M";
    sizeSelect.dispatchEvent(new Event("change", { bubbles: true }));
    await settleDomEvents();

    vi.mocked(bridge.addValidatedVariantCart).mockRejectedValue({
      kind: "product_unavailable",
      message: "That product is no longer available in the demo inventory.",
    });
    root.querySelector<HTMLButtonElement>("#add-cart")!.click();
    await settleDomEvents();

    expect(bridge.searchPortfolioProductsPage).toHaveBeenCalledWith(
      "Help me choose a dress for tonight.",
      false,
    );
  });
});
