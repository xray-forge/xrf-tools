import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

import { DialogsEditorOpenForm } from "./components/DialogsEditorOpenForm";
import { DialogsEditorWorkspace } from "./components/DialogsEditorWorkspace";

/** Picker until a project is open, editor once it is. */
export function DialogsEditorTool(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  return (
    <SessionTool
      root={dialogsService.project}
      openForm={<DialogsEditorOpenForm />}
      workspace={<DialogsEditorWorkspace />}
    />
  );
}
