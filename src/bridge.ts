import { invoke } from "@tauri-apps/api/core";

import type {
  CartSnapshot,
  ConciergeBridge,
  ProductChatContext,
  RecommendationOutcome,
} from "./contracts";

export function createTauriConciergeBridge(): ConciergeBridge {
  return {
    configureSessionOpenaiKey: (apiKey) =>
      invoke<{ concierge_enabled: boolean }>("configure_session_openai_key", { apiKey }),
    loadInitialProductTrio: () =>
      invoke<RecommendationOutcome>("load_initial_product_trio"),
    searchPortfolioProductsPage: (brief, showNextPage) =>
      invoke<RecommendationOutcome>("search_portfolio_products_page", { brief, showNextPage }),
    selectProductChatContext: (productSku, selectionSource) =>
      invoke<ProductChatContext>("select_product_chat_context", {
        productSku,
        selectionSource,
      }),
    updateProductVariantSelection: (productSku, selectedSize) =>
      invoke<void>("update_product_variant_selection", { productSku, selectedSize }),
    addValidatedVariantCart: (productSku, selectedSize) =>
      invoke<CartSnapshot>("add_validated_variant_cart", { productSku, selectedSize }),
  };
}
