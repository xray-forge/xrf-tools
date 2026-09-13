import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { DialogsService, IDialogSelection } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor } from "@/core/ipc/types/xrf-dialog";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { AsyncState } from "@/lib/async-state";
import { inline } from "@/lib/callbacks/inline";
import { Nullable } from "@/lib/types/general";

import { DialogGraph } from "./editor/DialogGraph";

/**
 * The centre of the editor: the graph, and nothing competing with it for room.
 */
export function DialogsEditorWorkspace(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const dialog: AsyncState<Nullable<DialogDescriptor>> = dialogsService.dialog;
  const selection: Nullable<IDialogSelection> = dialogsService.selection;

  if (!selection) {
    return (
      <EmptyState
        title={"No dialog selected"}
        description={"Pick one from the tree to see its phrases and where they lead."}
      />
    );
  }

  return (
    <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}>
      <EditorFileHeader
        data-testid={"dialog-header"}
        name={selection.id}
        caption={selection.logicalPath}
        closeLabel={"Close dialog"}
        closeDescription={"Clear the selection and close this dialog"}
        onClose={dialogsService.clearSelection}
      />

      <Box sx={{ display: "flex", flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}>
        {inline(() => {
          if (dialog.value) {
            return <DialogGraph dialog={dialog.value} onSelect={dialogsService.inspectNode} />;
          }

          if (dialog.isLoading) {
            return <DelayedProgress label={"Reading dialog…"} />;
          }

          return <ErrorState title={"Could not read this dialog"} description={String(dialog.error)} />;
        })}
      </Box>
    </Box>
  );
}
