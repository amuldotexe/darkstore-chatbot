export type ConciergeResult<Card = string> = {
  kind: "cards";
  requestId: string;
  cards: Card[];
  showNextThree: boolean;
};

export type ConciergeState<Card = string> = {
  latestRequestId: string | null;
  visibleCards: Card[];
  showNextThree: boolean;
};

export function createInitialConciergeState<Card>(): ConciergeState<Card> {
  return {
    latestRequestId: null,
    visibleCards: [],
    showNextThree: false,
  };
}

export function beginLatestRecommendationRequest<Card>(
  state: ConciergeState<Card>,
  requestId: string,
): ConciergeState<Card> {
  return {
    ...state,
    latestRequestId: requestId,
  };
}

export function applyLatestRecommendationOnly<Card>(
  state: ConciergeState<Card>,
  result: ConciergeResult<Card>,
): ConciergeState<Card> {
  if (state.latestRequestId !== result.requestId) {
    return state;
  }

  return {
    latestRequestId: result.requestId,
    visibleCards: result.cards,
    showNextThree: result.showNextThree,
  };
}
