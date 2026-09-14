import { IChoiceFormRowOption } from "@/core/ui/form";

/** Which of the two things the picker is opening. */
export enum EEquipmentOpenMode {
  /** A game folder, read as the engine mounts it: its archives, and the gamedata tree standing in front of them. */
  GAME = "game",
  /** One sheet file and the configuration beside it, for a sheet that is in no game tree. */
  SPRITE = "sprite",
}

/** Widest first: the whole game, then one sheet. */
export const OPEN_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EEquipmentOpenMode>> = [
  { value: EEquipmentOpenMode.GAME, label: "Game", "aria-label": "Open game" },
  { value: EEquipmentOpenMode.SPRITE, label: "Sprite", "aria-label": "Open sprite" },
];

export const OPEN_MODES: ReadonlyArray<EEquipmentOpenMode> = OPEN_MODE_OPTIONS.map(
  (option: IChoiceFormRowOption<EEquipmentOpenMode>) => option.value
);

/** What each mode promises, so the description says which question the mode answers rather than which files it reads. */
export const OPEN_MODE_DESCRIPTIONS: Readonly<Record<EEquipmentOpenMode, string>> = {
  [EEquipmentOpenMode.GAME]:
    "Finds the sheet and the configuration naming its icons the way the engine does, through the game's archives and " +
    "the loose gamedata tree standing in front of them. Nothing is written.",
  [EEquipmentOpenMode.SPRITE]:
    "Reads one sheet and the configuration beside it, for a sheet that is not in a game tree. Nothing is written.",
};

/** What the engine binds the inventory sheet by, hardcoded in `UIInventoryUtilities.cpp`. */
export const DEFAULT_SHEET_REFERENCE: string = "ui\\ui_icon_equipment";

/** Where the configs of a game tree live, which is fixed by `fsgame.ltx` in every install. */
export const GAME_CONFIG_PATH: string = "configs\\system.ltx";
