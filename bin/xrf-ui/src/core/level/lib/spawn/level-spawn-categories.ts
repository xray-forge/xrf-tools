import { ELevelSpawnCategory, LevelSpawnCategory } from "@/core/ipc/types/xrf-app";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";

/** The view options that show each category of spawned objects. */
export type TLevelSpawnOption = "isSpawnedProps" | "isSpawnedItems" | "isSpawnedWeapons" | "isSpawnedLamps";

/** One category of spawned objects as the viewer names and switches it. */
export interface ILevelSpawnCategoryEntry {
  category: ELevelSpawnCategory;
  label: string;
  /** The view option that shows it, which the toolbar and the panel both switch. */
  option: TLevelSpawnOption;
}

/** Every category, in the order the viewer lists them. */
export const LEVEL_SPAWN_CATEGORIES: ReadonlyArray<ILevelSpawnCategoryEntry> = [
  { category: ELevelSpawnCategory.PROPS, label: "Props", option: "isSpawnedProps" },
  { category: ELevelSpawnCategory.ITEMS, label: "Items", option: "isSpawnedItems" },
  { category: ELevelSpawnCategory.WEAPONS, label: "Weapons", option: "isSpawnedWeapons" },
  { category: ELevelSpawnCategory.LAMPS, label: "Lamps", option: "isSpawnedLamps" },
];

/**
 * @param options - What the toolbar has switched on.
 * @returns The categories of spawned objects shown.
 */
export function toLevelSpawnVisibility(options: ILevelViewOptions): ReadonlySet<LevelSpawnCategory> {
  return new Set(
    LEVEL_SPAWN_CATEGORIES.filter((entry: ILevelSpawnCategoryEntry) => options[entry.option]).map(
      (entry: ILevelSpawnCategoryEntry) => entry.category
    )
  );
}
