import { ELevelObjectKind, LevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";
import { Maybe, Nullable } from "@xrf/types";

/** The view options that show each kind of level object drawn as a model. */
export type TLevelVisibleOption = "isSpawnedProps" | "isSpawnedItems" | "isSpawnedWeapons" | "isSpawnedLamps";

/** One kind drawn as a model, and the view option its Visible switch is. */
export interface ILevelVisibleKindEntry {
  kind: LevelObjectKind;
  /** The view option that shows it, which the toolbar and the Outliner both switch. */
  option: TLevelVisibleOption;
}

/** Visible switches turned together, and whether their kinds are drawn from now on. */
export interface ILevelVisibleOptionsChange {
  options: ReadonlyArray<TLevelVisibleOption>;
  isOn: boolean;
}

/** Every kind drawn as a model, in the kind table's order. */
export const LEVEL_VISIBLE_KINDS: ReadonlyArray<ILevelVisibleKindEntry> = [
  { kind: ELevelObjectKind.PROPS, option: "isSpawnedProps" },
  { kind: ELevelObjectKind.ITEMS, option: "isSpawnedItems" },
  { kind: ELevelObjectKind.WEAPONS, option: "isSpawnedWeapons" },
  { kind: ELevelObjectKind.LAMPS, option: "isSpawnedLamps" },
];

/**
 * @param kind - A kind of level object.
 * @returns The view option its Visible switch is, or null for a kind with no picture of its own.
 */
export function getLevelVisibleOption(kind: LevelObjectKind): Nullable<TLevelVisibleOption> {
  const entry: Maybe<ILevelVisibleKindEntry> = LEVEL_VISIBLE_KINDS.find(
    (it: ILevelVisibleKindEntry) => it.kind === kind
  );

  return entry?.option ?? null;
}
