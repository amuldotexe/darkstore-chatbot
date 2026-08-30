import { describe, expect, it } from "vitest";

import {
  applyLatestRecommendationOnly,
  beginLatestRecommendationRequest,
  createInitialConciergeState,
  type ConciergeResult,
} from "./state";

describe("TEST-FRONTEND-RACE-009 recommendation reducer", () => {
  it("keeps a late result from replacing the newer request", () => {
    const state = createInitialConciergeState();
    const newResult: ConciergeResult = {
      kind: "cards",
      requestId: "newest",
      cards: ["SKID00083927", "SKID00174036", "SKID00081801"],
      showNextThree: true,
    };
    const lateResult: ConciergeResult = {
      kind: "cards",
      requestId: "older",
      cards: ["SKID00167395", "SKID00076560", "SKID00184392"],
      showNextThree: false,
    };

    const pendingNewest = beginLatestRecommendationRequest(state, "newest");
    const afterNewest = applyLatestRecommendationOnly(pendingNewest, newResult);
    const afterLate = applyLatestRecommendationOnly(afterNewest, lateResult);

    expect(afterLate.visibleCards).toEqual(newResult.cards);
    expect(afterLate.showNextThree).toBe(true);
  });
});
