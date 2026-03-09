/** Coordinates and dimensions of a screen capture region. */
export interface CaptureRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** A single stat extracted from OCR text. */
export interface ParsedStat {
  /** Stat label, e.g. "Force", "Intelligence", "% Résistance Feu". */
  name: string;
  /** Numeric value. For range patterns the midpoint is returned. */
  value: number;
  /** `true` when the stat has a negative prefix ("-"). */
  isNegative: boolean;
}

/** Structured OCR result for a Dofus item tooltip. */
export interface ParsedItemStats {
  /** Item name when detected as the first non-stat line, `null` otherwise. */
  itemName: string | null;
  /** All recognised stat entries. */
  stats: ParsedStat[];
}
