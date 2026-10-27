import { BaseComponentProps } from "@xrf/ui/element-types";
import { TGraphNode } from "@xrf/ui/graph/graph.types";
import { NodeProps } from "@xyflow/react";
import { ReactElement } from "react";

import { IDialogGraphNodeData } from "@/applications/dialogs-editor/lib";

import { DialogGraphNodeFrame } from "./DialogGraphNodeFrame";

/**
 * The accent a phrase draws in, which states which of three conditions its line is in.
 */
function toPhraseAccent(data: IDialogGraphNodeData): string {
  if (data.isUnresolved) {
    return "warning.main";
  }

  // Dimmed rather than hidden. It is why the node sits away from the conversation instead of in it.
  return data.hasIncoming ? "divider" : "text.disabled";
}

/** One phrase: the line the player reads, and the behaviour selecting it carries. */
export function DialogPhraseGraphNode({
  "data-testid": dataTestId = "dialog-phrase-graph-node",
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
      accent={toPhraseAccent(data)}
      hasSource
      hasTarget
    />
  );
}
