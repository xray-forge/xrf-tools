import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { DialogGraph } from "@/applications/dialogs-editor/components/editor/DialogGraph";
import { DialogLanguageBar } from "@/applications/dialogs-editor/components/editor/DialogLanguageBar";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor } from "@/core/bindings/types/xrf-dialog";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { Loadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

/**
 * What the centre says when it has no graph to draw.
 *
 * Read off the resource's own three states rather than off the selection, because a selection says
 * only that something was asked for. A failed read leaves one set with no value and no loading, which
 * a selection-driven branch reports as still fetching — forever, and without ever showing the error.
 */
function describeAbsentDialog(dialog: Loadable<Nullable<DialogDescriptor>>): { title: string; description: string } {
  if (dialog.isLoading) {
    return { description: "Fetching its phrases.", title: "Reading dialog" };
  }

  if (dialog.error) {
    return { description: String(dialog.error), title: "Could not read this dialog" };
  }

  return {
    description: "Pick one from the tree to see its phrases and where they lead.",
    title: "No dialog selected",
  };
}

/**
 * The centre of the editor: which language the lines are read in, and the graph itself.
 */
export function DialogsEditorWorkspace(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const dialog: Loadable<Nullable<DialogDescriptor>> = dialogsService.dialog;

  return (
    <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minHeight: 0, minWidth: 0 }}>
      <DialogLanguageBar
        languages={dialogsService.languages}
        selected={dialogsService.resolvedLanguage}
        onSelect={dialogsService.setLanguage}
      />

      {dialog.value ? (
        <DialogGraph dialog={dialog.value} onSelect={dialogsService.inspectNode} />
      ) : (
        <EmptyState {...describeAbsentDialog(dialog)} />
      )}
    </Box>
  );
}
