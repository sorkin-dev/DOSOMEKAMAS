import type { Rune } from "./rune";

/** Result of a probability calculation */
export interface ProbabilityResult {
  successRate: number;           // 0.0 to 1.0
  criticalSuccessRate: number;
  neutralFailureRate: number;
  failureRate: number;
  criticalFailureRate: number;
  expectedValue: number;         // EV in stat weight units
  expectedKamaCost: number;      // EV in kamas
  confidenceInterval: ConfidenceInterval;
  sampleSize: number;            // if based on historical data
}

export interface ConfidenceInterval {
  lower: number;
  upper: number;
  confidenceLevel: number; // e.g., 0.95
}

/** Recommendation from the optimization engine */
export interface ForgeRecommendation {
  recommendedRune: Rune;
  expectedOutcome: ProbabilityResult;
  reasoning: string;
  alternativeRunes: AlternativeRune[];
  currentSinkState: SinkState;
}

export interface AlternativeRune {
  rune: Rune;
  expectedOutcome: ProbabilityResult;
  tradeoffDescription: string;
}

export interface SinkState {
  currentSink: number;
  maxSink: number;
  sinkPercentage: number;           // 0.0 to 1.0
  estimatedSinkFromFailure: number;
  isSinkFavorable: boolean;         // true if sink state is advantageous
}

