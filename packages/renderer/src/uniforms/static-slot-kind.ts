/** What a slot draws, a flag of its record's fourth word. */
export enum EStaticSlotKind {
  /** Nothing: a slot free or drawing no index. */
  NONE = 0,
  /** Its clusters once, where its place puts them, each tested by its own sphere. */
  SINGLE = 1,
  /** Its clusters in every place a row of it keeps. */
  LISTED = 2,
}
