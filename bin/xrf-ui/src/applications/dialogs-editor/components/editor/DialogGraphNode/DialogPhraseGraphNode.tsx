import { NodeProps } from "@xyflow/react";
import { ReactElement } from "react";

import { DialogGraphNodeFrame } from "@/applications/dialogs-editor/components/editor/DialogGraphNode/DialogGraphNodeFrame";
import { IDialogGraphNodeData } from "@/applications/dialogs-editor/lib";
import { TGraphNode } from "@/core/graph/lib";

/**
 * The accent a phrase draws in, which states which of three conditions its line is in.
 *
 * Ordered by what a reader needs to act on first: text missing for this language outranks a phrase
 * nothing leads to, because one is work to do and the other is a question for validation.
 */
function toPhraseAccent(data: IDialogGraphNodeData): string {
  if (data.isUnresolved) {
    return "warning.main";
  }

  // Dimmed rather than hidden. It is why the node sits away from the conversation instead of in it.
  return data.hasIncoming ? "divider" : "text.disabled";
}

/** One phrase: the line the player reads, and the behaviour selecting it carries. */
export function DialogPhraseGraphNode({ data, selected }: NodeProps<TGraphNode<IDialogGraphNodeData>>): ReactElement {
  return (
    <DialogGraphNodeFrame
      data={data}
      isSelected={Boolean(selected)}
      accent={toPhraseAccent(data)}
      hasSource
      hasTarget
    />
  );
}
