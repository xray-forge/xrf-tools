import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { DialogDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { DialogViewService, IDialogSelection } from "@/applications/dialogs-editor/services/dialog-view";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { inline } from "@/lib/callbacks/inline";

import { DialogGraph } from "./editor/DialogGraph";

/**
 * The centre of the editor: the graph, and nothing competing with it for room.
 */
export function DialogsEditorWorkspace(): ReactElement {
  const viewService: DialogViewService = useInjection(DialogViewService);

  const dialog: AsyncState<Nullable<DialogDescriptor>> = viewService.dialog;
  const selection: Nullable<IDialogSelection> = viewService.selection;

  if (!selection) {
    return (
      <EmptyState
        title={"No dialog selected"}
        description={"Pick one from the tree to see its phrases and where they lead."}
      />
    );
  }

  return (
    <div className={"flex min-h-0 min-w-0 grow flex-col"}>
      <EditorFileHeader
        data-testid={"dialog-header"}
        name={selection.id}
        caption={selection.logicalPath}
        closeLabel={"Close dialog"}
        closeDescription={"Clear the selection and close this dialog"}
        onClose={viewService.clearSelection}
      />

      <div className={"flex min-h-0 min-w-0 grow overflow-hidden"}>
        {inline(() => {
          if (dialog.value) {
            return <DialogGraph dialog={dialog.value} onSelect={viewService.inspectNode} />;
          }

          if (dialog.isLoading) {
            return <DelayedProgress label={"Reading dialog…"} />;
          }

          return <ErrorState title={"Could not read this dialog"} description={String(dialog.error)} />;
        })}
      </div>
    </div>
  );
}
