import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { DialogsEditor } from "@/applications/dialogs-editor/components/DialogsEditor";
import { DialogsEditorOpenForm } from "@/applications/dialogs-editor/components/DialogsEditorOpenForm";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";

/** Picker until a project is open, editor once it is. */
export function DialogsEditorApplication(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  if (dialogsService.isReady) {
    return dialogsService.project.value ? <DialogsEditor /> : <DialogsEditorOpenForm />;
  }

  return <ApplicationLoader />;
}
