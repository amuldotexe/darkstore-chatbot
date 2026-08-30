import "../src/styles.css";

import { createConciergeApplication } from "../src/app";
import type {
  CartSnapshot,
  ConciergeBridge,
  ProductCard,
  ProductChatContext,
  RecommendationOutcome,
} from "../src/contracts";

const fixtureCards: ProductCard[] = [
  {
    sku: "SKID00083927",
    category_id: "dresses",
    brand: "Slikk X Revolte",
    product_name: "Black Minimalist A-Line Evening Dress For Date Night",
    current_price_inr: 1230,
    fixture_available: true,
    fixture_sizes: ["S", "M", "L"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 94,
    fixture_dress_type: "evening_dress",
    fixture_style_tags: ["minimalist", "date_night", "a_line", "black"],
    source_product_url: "https://www.slikk.club/",
  },
  {
    sku: "SKID00174036",
    category_id: "dresses",
    brand: "MYWISHBAG",
    product_name: "Edgy Floral Ruched Black Party Bodycon Dress",
    current_price_inr: 1300,
    fixture_available: true,
    fixture_sizes: ["S", "M", "L"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 92,
    fixture_dress_type: "party_dress",
    fixture_style_tags: ["party", "bodycon", "ruched", "floral", "black"],
    source_product_url: "https://www.slikk.club/",
  },
  {
    sku: "SKID00081801",
    category_id: "dresses",
    brand: "Slikk X Revolte",
    product_name: "Black Ruched Tube Dress For Date Nights",
    current_price_inr: 432,
    fixture_available: true,
    fixture_sizes: ["XS", "S", "M"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 89,
    fixture_dress_type: "tube_dress",
    fixture_style_tags: ["date_night", "ruched", "tube", "black"],
    source_product_url: "https://www.slikk.club/",
  },
  {
    sku: "SKID00207435",
    category_id: "dresses",
    brand: "BrownButter",
    product_name: "Coquette Hourglass A-Line Party Dress",
    current_price_inr: 1499,
    fixture_available: true,
    fixture_sizes: ["M", "L"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 87,
    fixture_dress_type: "party_dress",
    fixture_style_tags: ["coquette", "party", "hourglass", "a_line"],
    source_product_url: "https://www.slikk.club/dresses/dresses/BrownButter/Coquette-Hourglass-A-Line-Party-Dress/SKID00207435",
  },
  {
    sku: "SKID00119053",
    category_id: "dresses",
    brand: "Slikk X Revolte",
    product_name: "Minimalist Knotted Babydoll Black A-Line Dress",
    current_price_inr: 711,
    fixture_available: true,
    fixture_sizes: ["S", "M"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 85,
    fixture_dress_type: "babydoll_dress",
    fixture_style_tags: ["minimalist", "babydoll", "a_line", "black"],
    source_product_url: "https://www.slikk.club/dresses/dresses/Slikk%20x%20Revolte/Minimalist-Knotted-Babydoll-Black-A-Line-Dress/SKID00119053",
  },
  {
    sku: "SKID00184392",
    category_id: "dresses",
    brand: "Fiorra",
    product_name: "Orange Linen Minimalist A-Line Shirt Dress",
    current_price_inr: 1619,
    fixture_available: true,
    fixture_sizes: ["S", "M", "L"],
    fixture_delivery_minutes: 50,
    fixture_propensity_score: 82,
    fixture_dress_type: "shirt_dress",
    fixture_style_tags: ["linen", "minimalist", "a_line", "orange"],
    source_product_url: "https://www.slikk.club/dresses/dresses/Fiorra/Orange-Linen-Minimalist-A-Line-Shirt-Dress/SKID00184392",
  },
];

const initialCards = fixtureCards.slice(0, 3);
const nextCards = fixtureCards.slice(3, 6);

function createCardsOutcome(cards: ProductCard[], showNextThree: boolean): RecommendationOutcome {
  return {
    kind: "cards",
    category_id: "dresses",
    rationale: "A fixture-grounded dress edit for this E2E run.",
    cards,
    show_next_three: showNextThree,
  };
}

function createFixtureJourneyBridge(): ConciergeBridge {
  let cartItems: CartSnapshot["items"] = [];

  return {
    async configureSessionOpenaiKey() {
      return { concierge_enabled: true };
    },
    async loadInitialProductTrio() {
      return createCardsOutcome(initialCards, false);
    },
    async searchPortfolioProductsPage(brief, showNextPage) {
      if (/shirt|shoe|jacket/i.test(brief)) {
        return {
          kind: "not_in_inventory",
          category_id: null,
          rationale: "This v001 demo currently carries dresses only.",
          cards: [],
          show_next_three: false,
        };
      }
      return showNextPage
        ? createCardsOutcome(nextCards, false)
        : createCardsOutcome(initialCards, true);
    },
    async selectProductChatContext(productSku, selectionSource): Promise<ProductChatContext> {
      const product = fixtureCards.find((candidate) => candidate.sku === productSku);
      if (!product) {
        throw new Error("Fixture product is unavailable.");
      }
      return { product, selection_source: selectionSource, retained_brief: "A black dress for a dinner date" };
    },
    async updateProductVariantSelection() {},
    async addValidatedVariantCart(productSku, selectedSize) {
      const product = fixtureCards.find((candidate) => candidate.sku === productSku);
      if (!product) {
        throw new Error("Fixture product is unavailable.");
      }
      cartItems = [{ product, selected_size: selectedSize }];
      return { item_count: cartItems.length, items: cartItems };
    },
  };
}

const root = document.querySelector<HTMLElement>("#app");
if (!root) {
  throw new Error("E2E fixture needs an app root.");
}

createConciergeApplication(root, createFixtureJourneyBridge()).mountApplicationScreen();
