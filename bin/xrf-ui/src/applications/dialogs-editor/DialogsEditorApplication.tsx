import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

import { DialogsEditor } from "./components/DialogsEditor";
import { DialogsEditorOpenForm } from "./components/DialogsEditorOpenForm";

/** Picker until a project is open, editor once it is. */
export function DialogsEditorApplication(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  if (dialogsService.isReady) {
    return dialogsService.project.value ? <DialogsEditor /> : <DialogsEditorOpenForm />;
  }

  return <ApplicationLoader />;
}
