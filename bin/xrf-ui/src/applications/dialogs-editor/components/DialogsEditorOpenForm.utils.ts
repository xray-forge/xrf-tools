import { DialogProjectMode } from "@/core/ipc/types/xrf-dialog";
import { IChoiceFormRowOption } from "@/core/ui/form";

export const MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<DialogProjectMode>> = [
  { value: "gamedata", label: "Game data" },
  { value: "source", label: "Project sources" },
];

export const MODE_DESCRIPTIONS: Record<DialogProjectMode, string> = {
  gamedata: "Dialogs under configs\\gameplay, their text under configs\\text, as the game ships them.",
  source: "Dialogs under configs\\gameplay, their text as multi-language JSON in translations.",
};
