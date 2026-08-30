export type ProductCard = {
  sku: string;
  category_id: string;
  brand: string;
  product_name: string;
  current_price_inr: number;
  fixture_available: boolean;
  fixture_sizes: string[];
  fixture_delivery_minutes: number;
  fixture_propensity_score: number;
  fixture_dress_type: string;
  fixture_style_tags: string[];
  source_product_url: string;
};

export type RecommendationOutcome = {
  kind: "cards" | "not_in_inventory";
  category_id: string | null;
  rationale: string;
  cards: ProductCard[];
  show_next_three: boolean;
};

export type ProductChatContext = {
  product: ProductCard;
  selection_source: string;
  retained_brief: string;
};

export type CartSnapshot = {
  item_count: number;
  items: Array<{
    product: ProductCard;
    selected_size: string;
  }>;
};

export type ConciergeBridge = {
  configureSessionOpenaiKey(apiKey: string): Promise<{ concierge_enabled: boolean }>;
  loadInitialProductTrio(): Promise<RecommendationOutcome>;
  searchPortfolioProductsPage(
    brief: string,
    showNextPage: boolean,
  ): Promise<RecommendationOutcome>;
  selectProductChatContext(productSku: string, selectionSource: string): Promise<ProductChatContext>;
  updateProductVariantSelection(productSku: string, selectedSize: string): Promise<void>;
  addValidatedVariantCart(productSku: string, selectedSize: string): Promise<CartSnapshot>;
};
