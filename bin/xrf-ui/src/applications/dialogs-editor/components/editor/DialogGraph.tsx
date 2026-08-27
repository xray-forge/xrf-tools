import { NodeTypes, useEdgesState, useNodesState } from "@xyflow/react";
import { ReactElement, useEffect, useMemo } from "react";

import {
  DialogGraphNode,
  DialogPhraseGraphNode,
} from "@/applications/dialogs-editor/components/editor/DialogGraphNode";
import { buildDialogGraph, EDialogGraphNodeType, IDialogGraph } from "@/applications/dialogs-editor/lib";
import { DialogDescriptor } from "@/core/bindings/types/xrf-dialog";
import { GraphCanvas } from "@/core/graph/components";
import { TGraphEdge, TGraphNode } from "@/core/graph/lib";

const NODE_TYPES: NodeTypes = {
  [EDialogGraphNodeType.DIALOG]: DialogGraphNode,
  [EDialogGraphNodeType.PHRASE]: DialogPhraseGraphNode,
};

export interface IDialogGraphProps {
  dialog: DialogDescriptor;
}

/**
 * One dialog as a layered graph.
 *
 * Positions are recomputed from the dialog on every build rather than kept, which is what makes
 * reopening one deterministic and removes any sidecar file, debounce or drift before editing exists.
 * The canvas still owns node state so dragging and selection work — that state is simply reseeded
 * whenever the dialog changes, so a hand-nudged node lasts until the next selection and no longer.
 *
 * Requires a dialog rather than tolerating none: the workspace draws a placeholder while there is
 * nothing to show, so an empty canvas here would be a second answer to a question already settled.
 */
export function DialogGraph({ dialog }: IDialogGraphProps): ReactElement {
  const graph: IDialogGraph = useMemo(() => buildDialogGraph(dialog), [dialog]);

  const [nodes, setNodes, onNodesChange] = useNodesState<TGraphNode>(graph.nodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState<TGraphEdge>(graph.edges);

  // Reseeded on every rebuild. Without this the hooks keep their first graph and switching dialogs
  // draws the previous one, because `useNodesState` treats its argument as an initial value.
  useEffect(() => {
    setNodes(graph.nodes);
    setEdges(graph.edges);
  }, [graph, setEdges, setNodes]);

  return (
    <GraphCanvas
      data-testid={"dialog-graph"}
      nodes={nodes}
      edges={edges}
      nodeTypes={NODE_TYPES}
      isConnectable={false}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
    />
  );
}
