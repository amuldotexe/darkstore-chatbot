import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

import { createTauriConciergeBridge } from "./bridge";

describe("TEST-FRONTEND-IPC typed bridge", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("uses centralized command names and camel-cased owned payloads", async () => {
    invokeMock.mockResolvedValue({ concierge_enabled: true });
    const bridge = createTauriConciergeBridge();

    await bridge.configureSessionOpenaiKey("sk-test-key-that-is-never-sent");
    await bridge.searchPortfolioProductsPage("black dress", true);
    await bridge.addValidatedVariantCart("SKID00083927", "M");

    expect(invokeMock).toHaveBeenNthCalledWith(1, "configure_session_openai_key", {
      apiKey: "sk-test-key-that-is-never-sent",
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "search_portfolio_products_page", {
      brief: "black dress",
      showNextPage: true,
    });
    expect(invokeMock).toHaveBeenNthCalledWith(3, "add_validated_variant_cart", {
      productSku: "SKID00083927",
      selectedSize: "M",
    });
  });
});
