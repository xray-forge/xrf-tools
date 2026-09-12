import { DialogFilter } from "@tauri-apps/plugin-dialog";

import { IChoiceFormRowOption } from "@/core/ui/form";

/** Which of the two things the picker is opening. */
export const enum EArchiveOpenMode {
  DIRECTORY = "directory",
  ARCHIVE = "archive",
}

export const OPEN_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EArchiveOpenMode>> = [
  { value: EArchiveOpenMode.DIRECTORY, label: "Directory", "aria-label": "Open directory" },
  { value: EArchiveOpenMode.ARCHIVE, label: "Archive", "aria-label": "Open archive" },
];

export const OPEN_MODES: ReadonlyArray<EArchiveOpenMode> = OPEN_MODE_OPTIONS.map((option) => option.value);

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
