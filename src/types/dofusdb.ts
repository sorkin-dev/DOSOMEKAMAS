/** DofusDB API response types */

export interface DofusDbItem {
  id: number;
  name: LocalizedString;
  level: number;
  typeId: number;
  typeName: LocalizedString;
  iconUrl: string;
  description: LocalizedString;
  stats: DofusDbItemStat[];
  setId: number | null;
  setName: LocalizedString | null;
  conditions: string[];
  recipeIds: number[];
}

export interface LocalizedString {
  fr: string;
  en: string;
  es: string;
  pt: string;
  de: string;
  it: string;
}

export interface DofusDbItemStat {
  statId: number;
  name: LocalizedString;
  min: number;
  max: number;
  order: number;
}

export interface DofusDbRuneInfo {
  runeId: number;
  name: LocalizedString;
  statId: number;
  statName: LocalizedString;
  weight: number;
  valueBa: number;
  valuePa: number;
  valueRa: number;
}

/** Paginated response wrapper from DofusDB */
export interface DofusDbPaginatedResponse<T> {
  data: T[];
  total: number;
  skip: number;
  limit: number;
}

/** API error */
export interface DofusDbApiError {
  code: number;
  message: string;
  details: string | null;
}
