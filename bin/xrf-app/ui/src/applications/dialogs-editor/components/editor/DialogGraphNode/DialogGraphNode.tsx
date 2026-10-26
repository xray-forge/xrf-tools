import { NodeProps } from "@xyflow/react";
import { ReactElement } from "react";

import { IDialogGraphNodeData } from "@/applications/dialogs-editor/lib";
import { TGraphNode } from "@/core/graph/lib";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { DialogGraphNodeFrame } from "./DialogGraphNodeFrame";

/**
 * The dialog itself: its id, and the conditions gating the whole conversation.
 *
 * Draws no target handle, because a conversation starts here. Nothing in the graph leads to it.
 */
export function DialogGraphNode({
  "data-testid": dataTestId = "dialog-graph-node",
  id,
  className,
  data,
  selected,
}: NodeProps<TGraphNode<IDialogGraphNodeData>> & BaseComponentProps): ReactElement {
  return (
    <DialogGraphNodeFrame
      data-testid={dataTestId}
      id={id}
      className={className}
      data={data}
      isSelected={Boolean(selected)}
      accent={"primary.main"}
      hasSource
      hasTarget={false}
    />
  );
}
