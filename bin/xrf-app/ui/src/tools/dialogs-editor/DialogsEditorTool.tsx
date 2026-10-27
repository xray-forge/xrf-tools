import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/loading/ToolLoader";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

import { DialogsEditor } from "./components/DialogsEditor";
import { DialogsEditorOpenForm } from "./components/DialogsEditorOpenForm";

/** Picker until a project is open, editor once it is. */
export function DialogsEditorTool(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  if (dialogsService.isReady) {
    return dialogsService.project.value ? <DialogsEditor /> : <DialogsEditorOpenForm />;
  }

  return <ToolLoader />;
}
