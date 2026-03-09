/** Rune tier */
export enum RuneTier {
  Ba = "ba",  // basic rune
  Pa = "pa",  // medium rune (x3)
  Ra = "ra",  // powerful rune (x10)
}

/** Represents a forgemagie rune */
export interface Rune {
  runeId: string;
  name: string;
  statId: string;          // ID of the stat this rune affects
  statName: string;
  tier: RuneTier;
  flatValue: number;       // flat stat value added by this rune
  weight: number;          // forgemagie weight of this rune
  priceEstimate: number;   // estimated market price in kamas
  lastPriceUpdate: number; // timestamp ms
}

/** Rune application outcome */
export enum RuneOutcome {
  CriticalSuccess = "critical_success",  // over-success
  Success = "success",
  NeutralFailure = "neutral_failure",    // no change
  Failure = "failure",                   // stat lost, sink gained
  CriticalFailure = "critical_failure",  // stat lost, negative effect
}

/** Result of applying a rune */
export interface RuneApplicationResult {
  rune: Rune;
  outcome: RuneOutcome;
  statChanges: StatChange[];
  sinkDelta: number;
  timestamp: number;
}

export interface StatChange {
  statId: string;
  previousValue: number;
  newValue: number;
  delta: number;
}
