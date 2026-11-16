import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { DialogDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { Nullable } from "@xrf/types";
import { EditorFileHeader } from "@xrf/ui/editor/file-header";
import { ContentState } from "@xrf/ui/layout/content-state";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { ReactElement } from "react";

import { DialogViewService, IDialogSelection } from "@/tools/dialogs-editor/services/dialog-view";

import { DialogGraph } from "./DialogGraph";

/**
 * The centre of the editor: the graph, and nothing competing with it for room.
 */
export function DialogGraphView(): ReactElement {
  const viewService: DialogViewService = useInjection(DialogViewService);

  const dialog: AsyncState<DialogDescriptor> = viewService.dialog;
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
        <ContentState
          value={dialog.value}
          isLoading={dialog.isLoading}
          error={dialog.error}
          loadingLabel={"Reading dialog…"}
          errorTitle={"Could not read this dialog"}
          emptyTitle={"Dialog unavailable"}
          emptyDescription={"This dialog could not be read."}
        >
          {(value: DialogDescriptor) => (
            <DialogGraph
              dialog={value}
              selectedNodeId={viewService.inspectedNodeId}
              onSelect={viewService.inspectNode}
            />
          )}
        </ContentState>
      </div>
    </div>
  );
}
