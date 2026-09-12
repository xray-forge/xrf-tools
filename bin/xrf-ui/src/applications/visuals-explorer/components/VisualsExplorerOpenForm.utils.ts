import { IChoiceFormRowOption } from "@/core/ui/form";

/** Which of the two things the picker is opening. */
export const enum EVisualOpenMode {
  FOLDER = "folder",
  MODEL = "model",
}

export const OPEN_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EVisualOpenMode>> = [
  { value: EVisualOpenMode.FOLDER, label: "Folder", "aria-label": "Open folder" },
  { value: EVisualOpenMode.MODEL, label: "Model", "aria-label": "Open model" },
];

export const OPEN_MODES: ReadonlyArray<EVisualOpenMode> = OPEN_MODE_OPTIONS.map((option) => option.value);
