import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { DialogGraph } from "@/applications/dialogs-editor/components/editor/DialogGraph";
import { DialogLanguageBar } from "@/applications/dialogs-editor/components/editor/DialogLanguageBar";
import { DialogsTreeMenu } from "@/applications/dialogs-editor/components/editor/DialogsTreeMenu";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor, DialogProjectDescriptor } from "@/core/bindings/types/xrf-dialog";
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
function describeAbsentDialog(dialog: Loadable<Nullable<DialogDescriptor>>): {
  title: string;
  description: string;
} {
  if (dialog.isLoading) {
    return { description: "Fetching its phrases.", title: "Reading dialog" };
  }

  if (dialog.error) {
    return { description: String(dialog.error), title: "Could not read this dialog" };
  }

  return {
    description: "Pick one from the list to see its phrases and where they lead.",
    title: "No dialog selected",
  };
}

export function DialogsEditorWorkspace(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const project: Nullable<DialogProjectDescriptor> = dialogsService.project.value;
  const dialog: Loadable<Nullable<DialogDescriptor>> = dialogsService.dialog;

  const onSelect = useCallback(
    (logicalPath: string, id: string) => void dialogsService.selectDialog(logicalPath, id),
    [dialogsService]
  );

  return (
    <Box sx={{ display: "flex", flexGrow: 1, minHeight: 0 }}>
      <DialogsTreeMenu files={project?.files ?? {}} selection={dialogsService.selection} onSelect={onSelect} />

      <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minHeight: 0, minWidth: 0 }}>
        <DialogLanguageBar
          languages={dialogsService.languages}
          selected={dialogsService.resolvedLanguage}
          onSelect={dialogsService.setLanguage}
        />

        {dialog.value ? <DialogGraph dialog={dialog.value} /> : <EmptyState {...describeAbsentDialog(dialog)} />}
      </Box>
    </Box>
  );
}
