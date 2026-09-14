import { DialogFilter } from "@tauri-apps/plugin-dialog";

import { ARCHIVE_VOLUME_FILE_EXTENSIONS } from "@/core/archive/lib";
import { IChoiceFormRowOption } from "@/core/ui/form";

/** Which of the three things the picker is opening. */
export enum EArchiveOpenMode {
  DIRECTORY = "directory",
  ARCHIVE = "archive",
  /** A game folder, read as the engine mounts it. */
  GAME = "game",
}

/** Widest first: the whole game, then a directory of volumes, then one volume. */
export const OPEN_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EArchiveOpenMode>> = [
  { value: EArchiveOpenMode.GAME, label: "Game", "aria-label": "Open game" },
  { value: EArchiveOpenMode.DIRECTORY, label: "Directory", "aria-label": "Open directory" },
  { value: EArchiveOpenMode.ARCHIVE, label: "Archive", "aria-label": "Open archive" },
];

export const OPEN_MODES: ReadonlyArray<EArchiveOpenMode> = OPEN_MODE_OPTIONS.map((option) => option.value);

/** What each mode promises, so the description says which question the mode answers rather than which files it reads. */
export const OPEN_MODE_DESCRIPTIONS: Readonly<Record<EArchiveOpenMode, string>> = {
  [EArchiveOpenMode.DIRECTORY]: "Indexes every archive in the directory for browsing.",
  [EArchiveOpenMode.ARCHIVE]: "Indexes one archive volume for browsing.",
  [EArchiveOpenMode.GAME]:
    "Indexes the game the way the engine mounts it: its archives, and the loose gamedata tree standing in front of " +
    "them.",
};

/** Volume extensions offered by the dialog. */
export const ARCHIVE_FILTERS: Array<DialogFilter> = [
  { name: "Archive volume", extensions: [...ARCHIVE_VOLUME_FILE_EXTENSIONS] },
  { name: "All files", extensions: ["*"] },
];
