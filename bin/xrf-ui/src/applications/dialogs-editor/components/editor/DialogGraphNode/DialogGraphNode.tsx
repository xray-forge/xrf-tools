import { NodeProps } from "@xyflow/react";
import { ReactElement } from "react";

import { DialogGraphNodeFrame } from "@/applications/dialogs-editor/components/editor/DialogGraphNode/DialogGraphNodeFrame";
import { IDialogGraphNodeData } from "@/applications/dialogs-editor/lib";
import { TGraphNode } from "@/core/graph/lib";

/**
 * The dialog itself: its id, and the conditions gating the whole conversation.
 *
 * Draws no target handle, because a conversation starts here. Nothing in the graph leads to it.
 */
export function DialogGraphNode({ data, selected }: NodeProps<TGraphNode<IDialogGraphNodeData>>): ReactElement {
  return (
    <DialogGraphNodeFrame
      data={data}
      isSelected={selected === true}
      accent={"primary.main"}
      hasSource
      hasTarget={false}
    />
  );
}
