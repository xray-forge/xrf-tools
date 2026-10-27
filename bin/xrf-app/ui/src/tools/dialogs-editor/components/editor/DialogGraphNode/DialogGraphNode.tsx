import { BaseComponentProps } from "@xrf/ui/element-types";
import { TGraphNode } from "@xrf/ui/graph";
import { NodeProps } from "@xyflow/react";
import { ReactElement } from "react";

import { IDialogGraphNodeData } from "@/tools/dialogs-editor/lib";

import { DialogGraphNodeFrame } from "./DialogGraphNodeFrame";

/**
 * The dialog itself: its id, and the conditions gating the whole conversation.
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
