import { DialogProjectMode } from "@xrf/ipc/types/xrf-dialog";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

export const MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<DialogProjectMode>> = [
  { value: "gamedata", label: "Game data" },
  { value: "source", label: "Project sources" },
];

export const MODE_DESCRIPTIONS: Record<DialogProjectMode, string> = {
  gamedata: "Dialogs under configs\\gameplay, their text under configs\\text, as the game ships them.",
  source: "Dialogs under configs\\gameplay, their text as multi-language JSON in translations.",
};
