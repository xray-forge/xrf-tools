import { DialogFilter } from "@tauri-apps/plugin-dialog";

import { IChoiceFormRowOption } from "@/core/ui/form";

/** Which of the three things the picker is opening. */
export const enum EArchiveOpenMode {
  DIRECTORY = "directory",
  ARCHIVE = "archive",
  /**  A game folder, read as the engine mounts it. */
  INSTALLATION = "installation",
}

export const OPEN_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EArchiveOpenMode>> = [
  { value: EArchiveOpenMode.DIRECTORY, label: "Directory", "aria-label": "Open directory" },
  { value: EArchiveOpenMode.ARCHIVE, label: "Archive", "aria-label": "Open archive" },
  { value: EArchiveOpenMode.INSTALLATION, label: "Installation", "aria-label": "Open installation" },
];

export const OPEN_MODES: ReadonlyArray<EArchiveOpenMode> = OPEN_MODE_OPTIONS.map((option) => option.value);

/** What each mode promises, so the description says which question the mode answers rather than which files it reads. */
export const OPEN_MODE_DESCRIPTIONS: Readonly<Record<EArchiveOpenMode, string>> = {
  [EArchiveOpenMode.DIRECTORY]: "Indexes every archive in the directory for browsing.",
  [EArchiveOpenMode.ARCHIVE]: "Indexes one archive volume for browsing.",
  [EArchiveOpenMode.INSTALLATION]:
    "Indexes the game the way the engine mounts it: its archives, and the loose gamedata tree standing in front of " +
    "them.",
};

/** Volume extensions offered by the dialog. */
export const ARCHIVE_FILTERS: Array<DialogFilter> = [
  {
    name: "Archive volume",
    extensions: ["db", "xdb"].flatMap((base: string) => [
      base,
      ...Array.from({ length: 10 }, (_, index: number) => `${base}${index}`),
    ]),
  },
  { name: "All files", extensions: ["*"] },
];
