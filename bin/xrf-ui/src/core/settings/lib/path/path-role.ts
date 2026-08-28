/**
 * What a tool is asking for, rather than where it happens to live.
 */
export enum EPathRole {
  /** Directory of LTX configuration files. */
  CONFIGS = "configs",
  /** `system.ltx`, which names every item a sprite can carry an icon for. */
  SYSTEM_LTX = "systemLtx",
  /** Root holding `configs` and the translations beside it, mounted whole. */
  CONTENT_ROOT = "contentRoot",
  /** Directory of translation sources. */
  TRANSLATIONS = "translations",
  /** Built `all.spawn`. */
  ALL_SPAWN = "allSpawn",
  /** Packed equipment icon sprite. */
  EQUIPMENT_SPRITE = "equipmentSprite",
  /** Directory of individual equipment icon images. */
  EQUIPMENT_ICON_SOURCES = "equipmentIconSources",
  /** Directory holding archive volumes to read. */
  ARCHIVES = "archives",
  /** The loose game data tree itself, as a source or as a destination. */
  GAMEDATA = "gamedata",
  /** Tree holding the translation XML the game loads, which an installation and game data both have. */
  BUILT_TRANSLATIONS = "builtTranslations",
  /** Tree of visual assets to browse. */
  VISUALS = "visuals",
}
