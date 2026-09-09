import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { DialogGraph } from "@/applications/dialogs-editor/components/editor/DialogGraph";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { DialogDescriptor } from "@/core/bindings/types/xrf-dialog";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { Loadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

/**
 * The centre of the editor: the graph, and nothing competing with it for room.
 */
export function DialogsEditorWorkspace(): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const dialog: Loadable<Nullable<DialogDescriptor>> = dialogsService.dialog;

  if (dialog.value) {
    return <DialogGraph dialog={dialog.value} onSelect={dialogsService.inspectNode} />;
  }

  if (dialog.isLoading) {
    return <DelayedProgress label={"Reading dialog…"} />;
  }

  if (dialog.error) {
    return <ErrorState title={"Could not read this dialog"} description={String(dialog.error)} />;
  }

  return (
    <EmptyState
      title={"No dialog selected"}
      description={"Pick one from the tree to see its phrases and where they lead."}
    />
  );
}
