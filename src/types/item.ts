/** Represents a single stat on an item */
export interface ItemStat {
  statId: string;
  name: string;
  currentValue: number;
  baseValue: number;      // base value on the item template
  minValue: number;       // minimum possible value for this stat
  maxValue: number;       // maximum possible (perfect) value
  isNegative: boolean;    // true if this stat is a malus
  weight: number;         // forgemagie weight of this stat
}

/** Represents the full state of an item being forgemaged */
export interface ItemState {
  itemId: number;
  name: string;
  level: number;
  category: ItemCategory;
  stats: ItemStat[];
  sinkValue: number;       // current puits (sink) value
  maxSinkCapacity: number; // max sink the item can hold
  isIdentified: boolean;   // whether item has been parsed via OCR
  lastUpdated: number;     // timestamp ms
}

export enum ItemCategory {
  Weapon = "weapon",
  Hat = "hat",
  Cloak = "cloak",
  Belt = "belt",
  Boots = "boots",
  Ring = "ring",
  Amulet = "amulet",
  Shield = "shield",
  Dofus = "dofus",
  Trophy = "trophy",
  Pet = "pet",
  PetMount = "pet_mount",
  Mount = "mount",
}
