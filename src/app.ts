import type {
  ConciergeBridge,
  ProductCard,
  ProductChatContext,
  RecommendationOutcome,
} from "./contracts";
import {
  applyLatestRecommendationOnly,
  beginLatestRecommendationRequest,
  createInitialConciergeState,
  type ConciergeState,
} from "./state";

export type { ConciergeBridge, ProductCard } from "./contracts";

type ScreenName = "key" | "discover" | "chat";
type SearchMode = "hidden" | "entry";

type ShopperError = {
  kind: string;
  message: string;
};

export class ConciergeApplication {
  private screen: ScreenName = "key";
  private searchMode: SearchMode = "hidden";
  private recommendationState: ConciergeState<ProductCard> = createInitialConciergeState<ProductCard>();
  private recommendationOutcome: RecommendationOutcome | null = null;
  private selectedChat: ProductChatContext | null = null;
  private selectedSize = "";
  private cartCount = 0;
  private cartAdded = false;
  private requestSequence = 0;
  private statusMessage = "";
  private statusTone: "neutral" | "error" | "success" = "neutral";
  private chatMessages: Array<{ speaker: "shopper" | "concierge"; text: string }> = [];
  private detailOpen = false;
  private portfolioBrief = "";
  private portfolioSearchPending = false;

  public constructor(
    private readonly root: HTMLElement,
    private readonly bridge: ConciergeBridge,
  ) {}

  public mountApplicationScreen(): void {
    this.renderCurrentScreen();
  }

  private renderCurrentScreen(): void {
    if (this.screen === "key") {
      this.renderKeyGateScreen();
      return;
    }
    if (this.screen === "chat") {
      this.renderProductChatScreen();
      return;
    }
    this.renderDiscoveryScreen();
  }

  private renderKeyGateScreen(): void {
    this.root.innerHTML = `
      <section class="key-shell" aria-labelledby="key-title">
        <p class="eyebrow">DARK-STORE FASHION CONCIERGE</p>
        <h1 id="key-title">Three dresses. A fast, grounded edit.</h1>
        <p class="intro">Enter your own OpenAI key for this session. It stays in memory, never in this app’s files.</p>
        ${this.renderStatusMessage()}
        <form id="key-gate" class="key-form">
          <label for="api-key">OpenAI API key</label>
          <input id="api-key" name="api-key" type="password" autocomplete="off" spellcheck="false" required placeholder="sk-…" />
          <button class="primary-button" type="submit">Start your edit <span aria-hidden="true">→</span></button>
        </form>
        <p class="fine-print">v002 uses a small, dress-only demonstration catalogue. No checkout or payment data is collected.</p>
      </section>`;
    this.root.querySelector<HTMLFormElement>("#key-gate")?.addEventListener("submit", (event) => {
      event.preventDefault();
      void this.submitSessionKey();
    });
  }

  private renderDiscoveryScreen(): void {
    const outcome = this.recommendationOutcome;
    const productCards = this.portfolioSearchPending ? [] : outcome?.kind === "cards" ? outcome.cards : [];
    const isSearch = this.searchMode === "entry";
    const hasAbsence = outcome?.kind === "not_in_inventory";
    this.root.innerHTML = `
      <div class="application-shell">
        ${this.renderApplicationHeader()}
        <main class="discovery-layout" aria-live="polite">
          <section class="discovery-main" aria-labelledby="edit-title">
            <p class="eyebrow">${isSearch ? "NEW BRIEF" : "YOUR FIRST LOOK"}</p>
            <h1 id="edit-title">${isSearch ? "Tell me what you had in mind." : "A sharper place to start."}</h1>
            <p class="intro">${isSearch ? "Share a colour, mood, occasion, or style. The concierge selects only from the dresses available right now." : "Three available dresses selected for this demo edit—then one clear path to search again."}</p>
            ${this.renderStatusMessage()}
            ${isSearch || hasAbsence ? this.renderPortfolioSearchForm() : ""}
            ${this.portfolioSearchPending ? '<p class="search-pending" role="status">Finding matching dresses…</p>' : ""}
            ${hasAbsence ? this.renderAbsentCategoryMessage(outcome) : ""}
            ${productCards.length > 0 ? this.renderProductCardGrid(productCards) : ""}
          </section>
          <aside class="edit-sidebar" aria-label="How this demo works">
            <p class="eyebrow">THE V002 RULE</p>
            <p>GPT-4o chooses only from the available dress cards. Embedded catalogue facts validate every selection, with local propensity ranking as the safe fallback.</p>
            <dl>
              <div><dt>Catalog today</dt><dd>Dresses only</dd></div>
              <div><dt>Delivery fixture</dt><dd>50 min</dd></div>
              <div><dt>Cart</dt><dd>Local only</dd></div>
            </dl>
          </aside>
        </main>
      </div>`;

    this.bindDiscoveryActions();
  }

  private renderApplicationHeader(): string {
    return `
      <header class="application-header">
        <a class="wordmark" href="#first-look" id="return-first-look">Darkstore Concierge</a>
        <p>Fashion, made decisive.</p>
        <button class="cart-button" type="button" aria-label="Local cart has ${this.cartCount} item${this.cartCount === 1 ? "" : "s"}">Cart ${this.cartCount}</button>
      </header>`;
  }

  private renderPortfolioSearchForm(): string {
    return `
      <form id="portfolio-search" class="portfolio-form">
        <label for="portfolio-brief">What are you looking for?</label>
        <div>
          <input id="portfolio-brief" name="portfolio-brief" autocomplete="off" required value="${this.escapeHtml(this.portfolioBrief)}" placeholder="e.g. a black dress for a dinner date" />
          <button class="primary-button" type="submit" ${this.portfolioSearchPending ? "disabled" : ""}>${this.portfolioSearchPending ? "Finding matching dresses…" : "Show matching dresses"}</button>
        </div>
      </form>`;
  }

  private renderAbsentCategoryMessage(outcome: RecommendationOutcome): string {
    return `
      <section class="empty-state" role="status">
        <p class="eyebrow">NOT IN THIS DEMO</p>
        <h2>${this.escapeHtml(outcome.rationale)}</h2>
        <p>Refine the brief and I’ll look again. I won’t substitute a different product category.</p>
      </section>`;
  }

  private renderProductCardGrid(cards: ProductCard[]): string {
    const fourthAction = this.renderFourthDiscoveryCard();
    return `<section class="product-grid" aria-label="${cards.length} available recommendations">
      ${cards.map((card, index) => this.renderProductCard(card, index)).join("")}
      ${fourthAction}
    </section>`;
  }

  private renderFourthDiscoveryCard(): string {
    if (this.searchMode === "entry" && this.recommendationOutcome?.show_next_three) {
      return `<button class="fourth-card" id="next-three" type="button"><span>Show 3 more</span><small>See the next available dresses</small><b aria-hidden="true">→</b></button>`;
    }
    if (this.searchMode === "entry") {
      return `<aside class="fourth-card inventory-terminal" role="status"><span>No more inventory.</span><small>Try another brief above to start a new dress search.</small></aside>`;
    }
    return `<button class="fourth-card" id="something-else" type="button"><span>Search another dress</span><small>Describe a colour, mood, occasion, or style</small><b aria-hidden="true">→</b></button>`;
  }

  private renderProductCard(card: ProductCard, index: number): string {
    const tags = card.fixture_style_tags.slice(0, 2).map(this.escapeHtml).join(" · ");
    return `
      <article class="product-card" data-product-sku="${this.escapeHtml(card.sku)}">
        <div class="product-art art-${index + 1}" aria-hidden="true"><span>${this.escapeHtml(card.fixture_dress_type.replaceAll("_", " "))}</span></div>
        <div class="card-copy">
          <p class="brand">${this.escapeHtml(card.brand)}</p>
          <h2>${this.escapeHtml(card.product_name)}</h2>
          <p class="product-tags">${tags}</p>
          <div class="price-row"><strong>${this.formatIndianRupees(card.current_price_inr)}</strong><span>${card.fixture_delivery_minutes} min delivery</span></div>
          <p class="why-this">Edit signal ${card.fixture_propensity_score}/100 · available in ${card.fixture_sizes.join(", ")}</p>
          <button class="select-button" data-select-sku="${this.escapeHtml(card.sku)}" type="button">Choose this dress <span aria-hidden="true">→</span></button>
        </div>
      </article>`;
  }

  private renderProductChatScreen(): void {
    const selected = this.selectedChat;
    if (!selected) {
      this.screen = "discover";
      this.renderDiscoveryScreen();
      return;
    }
    const product = selected.product;
    const chatMessages = this.chatMessages
      .map(({ speaker, text }) => `<li class="message ${speaker}"><span>${speaker === "shopper" ? "You" : "Edit"}</span><p>${this.escapeHtml(text)}</p></li>`)
      .join("");
    const options = product.fixture_sizes
      .map((size) => `<option value="${this.escapeHtml(size)}" ${this.selectedSize === size ? "selected" : ""}>${this.escapeHtml(size)}</option>`)
      .join("");
    this.root.innerHTML = `
      <div class="application-shell">
        ${this.renderApplicationHeader()}
        <main class="chat-layout">
          <section class="product-summary" aria-labelledby="selected-product-title">
            <button id="chat-back" class="quiet-button" type="button">← Back to the edit</button>
            <div class="product-art art-1" aria-hidden="true"><span>${this.escapeHtml(product.fixture_dress_type.replaceAll("_", " "))}</span></div>
            <p class="brand">${this.escapeHtml(product.brand)}</p>
            <h1 id="selected-product-title">${this.escapeHtml(product.product_name)}</h1>
            <p class="price">${this.formatIndianRupees(product.current_price_inr)}</p>
            <a id="product-details" href="#product-details">Product details</a>
            ${this.detailOpen ? `<section class="detail-panel" id="product-detail-panel"><p>Sizes: ${product.fixture_sizes.map(this.escapeHtml).join(", ")}</p><p>Demo delivery: ${product.fixture_delivery_minutes} minutes.</p><p>Source record: ${this.escapeHtml(product.source_product_url)}</p></section>` : ""}
          </section>
          <section class="chat-panel" aria-label="Product customization chat">
            <p class="eyebrow">CUSTOMIZE IN CHAT</p>
            <h2>Make the ${this.escapeHtml(product.fixture_dress_type.replaceAll("_", " "))} yours.</h2>
            <p class="retained-brief">Your brief: ${this.escapeHtml(selected.retained_brief)}</p>
            <ol class="message-list">${chatMessages}</ol>
            <form id="customise-form" class="customise-form">
              <label for="customise-question">Ask about styling, fit or occasion</label>
              <div><input id="customise-question" autocomplete="off" placeholder="What shoes would work?" /><button type="submit">Ask</button></div>
            </form>
            ${this.renderStatusMessage()}
            <section class="cart-composer" aria-label="Choose size and add to cart">
              <label for="selected-size">Choose your size</label>
              <select id="selected-size"><option value="">Select a size</option>${options}</select>
              <button id="add-cart" class="primary-button" type="button" ${this.selectedSize && !this.cartAdded ? "" : "disabled"}>${this.cartAdded ? "Added to local cart" : "Add to local cart"}</button>
              <button id="chat-more" class="quiet-button" type="button">I want something else</button>
            </section>
          </section>
        </main>
      </div>`;
    this.bindChatActions();
  }

  private bindDiscoveryActions(): void {
    this.root.querySelector<HTMLAnchorElement>("#return-first-look")?.addEventListener("click", (event) => {
      event.preventDefault();
      void this.loadFirstLookRecommendations();
    });
    this.root.querySelector<HTMLFormElement>("#portfolio-search")?.addEventListener("submit", (event) => {
      event.preventDefault();
      const brief = this.root.querySelector<HTMLInputElement>("#portfolio-brief")?.value ?? "";
      void this.loadPortfolioRecommendations(brief, false);
    });
    this.root.querySelector<HTMLButtonElement>("#something-else")?.addEventListener("click", () => {
      this.enterPortfolioSearchMode();
      this.renderDiscoveryScreen();
      this.root.querySelector<HTMLInputElement>("#portfolio-brief")?.focus();
    });
    this.root.querySelector<HTMLButtonElement>("#next-three")?.addEventListener("click", () => {
      void this.loadPortfolioRecommendations(this.portfolioBrief, true);
    });
    this.root.querySelectorAll<HTMLButtonElement>("[data-select-sku]").forEach((button) => {
      button.addEventListener("click", () => {
        const productSku = button.dataset.selectSku;
        if (productSku) {
          const source = this.searchMode === "entry" ? "search_result" : "first_look";
          void this.selectProductForChat(productSku, source);
        }
      });
    });
  }

  private bindChatActions(): void {
    this.root.querySelector<HTMLButtonElement>("#chat-back")?.addEventListener("click", () => {
      this.screen = "discover";
      this.statusMessage = "";
      this.renderDiscoveryScreen();
    });
    this.root.querySelector<HTMLAnchorElement>("#product-details")?.addEventListener("click", (event) => {
      event.preventDefault();
      this.detailOpen = !this.detailOpen;
      this.renderProductChatScreen();
    });
    this.root.querySelector<HTMLFormElement>("#customise-form")?.addEventListener("submit", (event) => {
      event.preventDefault();
      const question = this.root.querySelector<HTMLInputElement>("#customise-question")?.value.trim() ?? "";
      if (question) {
        this.chatMessages.push({ speaker: "shopper", text: question });
        this.chatMessages.push({ speaker: "concierge", text: this.createFixtureStylingResponse() });
        this.renderProductChatScreen();
      }
    });
    this.root.querySelector<HTMLSelectElement>("#selected-size")?.addEventListener("change", (event) => {
      const selectedSize = (event.currentTarget as HTMLSelectElement).value;
      void this.selectProductSize(selectedSize);
    });
    this.root.querySelector<HTMLButtonElement>("#add-cart")?.addEventListener("click", () => {
      void this.addSelectedProductCart();
    });
    this.root.querySelector<HTMLButtonElement>("#chat-more")?.addEventListener("click", () => {
      this.screen = "discover";
      this.enterPortfolioSearchMode();
      this.renderDiscoveryScreen();
      this.root.querySelector<HTMLInputElement>("#portfolio-brief")?.focus();
    });
  }

  private async submitSessionKey(): Promise<void> {
    const apiKey = this.root.querySelector<HTMLInputElement>("#api-key")?.value ?? "";
    try {
      await this.bridge.configureSessionOpenaiKey(apiKey);
      this.screen = "discover";
      this.searchMode = "hidden";
      this.statusMessage = "";
      await this.loadFirstLookRecommendations();
    } catch (error) {
      this.setShopperError(error);
      this.renderKeyGateScreen();
    }
  }

  private async loadFirstLookRecommendations(): Promise<void> {
    this.searchMode = "hidden";
    this.portfolioBrief = "";
    this.portfolioSearchPending = false;
    this.recommendationOutcome = null;
    this.recommendationState = createInitialConciergeState<ProductCard>();
    const requestId = this.beginRecommendationRequest();
    try {
      const outcome = await this.bridge.loadInitialProductTrio();
      this.applyRecommendationOutcome(requestId, outcome);
    } catch (error) {
      this.applyRecommendationError(requestId, error);
    }
  }

  private async loadPortfolioRecommendations(brief: string, showNextPage: boolean): Promise<void> {
    const trimmedBrief = brief.trim();
    if (!trimmedBrief) {
      this.statusTone = "error";
      this.statusMessage = "Describe what you are looking for first.";
      this.renderDiscoveryScreen();
      return;
    }
    this.portfolioBrief = trimmedBrief;
    this.portfolioSearchPending = true;
    this.renderDiscoveryScreen();
    const requestId = this.beginRecommendationRequest();
    try {
      const outcome = await this.bridge.searchPortfolioProductsPage(trimmedBrief, showNextPage);
      this.applyRecommendationOutcome(requestId, outcome);
    } catch (error) {
      this.applyRecommendationError(requestId, error);
    }
  }

  private async selectProductForChat(productSku: string, selectionSource: string): Promise<void> {
    try {
      const selectedChat = await this.bridge.selectProductChatContext(productSku, selectionSource);
      this.selectedChat = selectedChat;
      this.selectedSize = "";
      this.cartAdded = false;
      this.detailOpen = false;
      this.statusMessage = "";
      this.chatMessages = [
        {
          speaker: "concierge",
          text: `This ${selectedChat.product.fixture_dress_type.replaceAll("_", " ")} is available in ${selectedChat.product.fixture_sizes.join(", ")}. Tell me the mood, fit, or occasion you want to shape.`,
        },
      ];
      this.screen = "chat";
      this.renderProductChatScreen();
    } catch (error) {
      this.setShopperError(error);
      this.renderDiscoveryScreen();
    }
  }

  private async selectProductSize(selectedSize: string): Promise<void> {
    const selectedProduct = this.selectedChat?.product;
    if (!selectedProduct) {
      return;
    }
    try {
      await this.bridge.updateProductVariantSelection(selectedProduct.sku, selectedSize);
      this.selectedSize = selectedSize;
      this.cartAdded = false;
      this.statusMessage = "";
    } catch (error) {
      this.selectedSize = "";
      this.setShopperError(error);
    }
    this.renderProductChatScreen();
  }

  private async addSelectedProductCart(): Promise<void> {
    const selectedProduct = this.selectedChat?.product;
    if (!selectedProduct || !this.selectedSize) {
      return;
    }
    try {
      const cart = await this.bridge.addValidatedVariantCart(selectedProduct.sku, this.selectedSize);
      this.cartCount = cart.item_count;
      this.cartAdded = true;
      this.statusTone = "success";
      this.statusMessage = `${selectedProduct.product_name} in ${this.selectedSize} is in your local cart.`;
    } catch (error) {
      const shopperError = this.normaliseShopperError(error);
      if (shopperError.kind === "product_unavailable") {
        const retainedBrief = this.selectedChat?.retained_brief;
        if (retainedBrief) {
          this.screen = "discover";
          this.searchMode = "entry";
          this.statusTone = "error";
          this.statusMessage = "That dress is no longer available. Here are complete alternatives for the same brief.";
          await this.loadPortfolioRecommendations(retainedBrief, false);
          return;
        }
      }
      this.setShopperError(error);
    }
    this.renderProductChatScreen();
  }

  private beginRecommendationRequest(): string {
    const requestId = `recommendation-${++this.requestSequence}`;
    this.recommendationState = beginLatestRecommendationRequest(this.recommendationState, requestId);
    this.statusMessage = "";
    return requestId;
  }

  private enterPortfolioSearchMode(): void {
    this.searchMode = "entry";
    this.statusMessage = "";
    this.portfolioBrief = "";
    this.portfolioSearchPending = false;
    this.recommendationOutcome = null;
    this.recommendationState = createInitialConciergeState<ProductCard>();
  }

  private applyRecommendationOutcome(requestId: string, outcome: RecommendationOutcome): void {
    if (this.recommendationState.latestRequestId !== requestId) {
      return;
    }
    this.recommendationState = applyLatestRecommendationOnly(this.recommendationState, {
      kind: "cards",
      requestId,
      cards: outcome.cards,
      showNextThree: outcome.show_next_three,
    });
    this.portfolioSearchPending = false;
    this.recommendationOutcome = outcome;
    this.statusMessage = "";
    this.renderDiscoveryScreen();
  }

  private applyRecommendationError(requestId: string, error: unknown): void {
    if (this.recommendationState.latestRequestId !== requestId) {
      return;
    }
    this.portfolioSearchPending = false;
    const shopperError = this.normaliseShopperError(error);
    if (shopperError.kind === "complete_page_exhausted" && this.applyExhaustedInventoryTerminal()) {
      this.renderDiscoveryScreen();
      return;
    }
    this.recommendationOutcome = null;
    this.statusTone = "error";
    this.statusMessage = shopperError.message;
    this.renderDiscoveryScreen();
  }

  private applyExhaustedInventoryTerminal(): boolean {
    if (this.searchMode !== "entry" || this.recommendationOutcome?.kind !== "cards") {
      return false;
    }
    this.recommendationOutcome = {
      ...this.recommendationOutcome,
      show_next_three: false,
    };
    this.recommendationState = {
      ...this.recommendationState,
      showNextThree: false,
    };
    this.statusTone = "neutral";
    this.statusMessage = "";
    return true;
  }

  private createFixtureStylingResponse(): string {
    const product = this.selectedChat?.product;
    if (!product) {
      return "Choose a dress first and I’ll keep the edit grounded.";
    }
    const tags = product.fixture_style_tags.join(", ");
    return `For this v002 edit, lean into ${tags}. The product facts here are fixed demo fixtures; choose a listed size before adding it to your local cart.`;
  }

  private setShopperError(error: unknown): void {
    const shopperError = this.normaliseShopperError(error);
    this.statusTone = "error";
    this.statusMessage = shopperError.message;
  }

  private normaliseShopperError(error: unknown): ShopperError {
    if (typeof error === "object" && error !== null && "message" in error) {
      const candidate = error as { kind?: unknown; message?: unknown };
      if (typeof candidate.message === "string") {
        return {
          kind: typeof candidate.kind === "string" ? candidate.kind : "command_error",
          message: candidate.message,
        };
      }
    }
    return {
      kind: "command_error",
      message: "Something went wrong. Please try again.",
    };
  }

  private renderStatusMessage(): string {
    if (!this.statusMessage) {
      return "";
    }
    return `<p class="status-message ${this.statusTone}" role="status">${this.escapeHtml(this.statusMessage)}</p>`;
  }

  private formatIndianRupees(price: number): string {
    return new Intl.NumberFormat("en-IN", {
      style: "currency",
      currency: "INR",
      maximumFractionDigits: 0,
    }).format(price);
  }

  private escapeHtml(value: string): string {
    return value.replace(/[&<>'"]/g, (character) => {
      const encoded: Record<string, string> = {
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        "'": "&#39;",
        '"': "&quot;",
      };
      return encoded[character] ?? character;
    });
  }
}

export function createConciergeApplication(
  root: HTMLElement,
  bridge: ConciergeBridge,
): ConciergeApplication {
  return new ConciergeApplication(root, bridge);
}
