import { NodeTypes, OnSelectionChangeParams, useEdgesState, useNodesState } from "@xyflow/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import {
  DialogGraphNode,
  DialogPhraseGraphNode,
} from "@/applications/dialogs-editor/components/editor/DialogGraphNode";
import { buildDialogGraph, EDialogGraphNodeType, IDialogGraph } from "@/applications/dialogs-editor/lib";
import { DialogDescriptor } from "@/core/bindings/types/xrf-dialog";
import { GraphCanvas } from "@/core/graph/components";
import { TGraphEdge, TGraphNode } from "@/core/graph/lib";
import { Nullable } from "@/lib/types/general";

const NODE_TYPES: NodeTypes = {
  [EDialogGraphNodeType.DIALOG]: DialogGraphNode,
  [EDialogGraphNodeType.PHRASE]: DialogPhraseGraphNode,
};

export interface IDialogGraphProps {
  dialog: DialogDescriptor;
  /** Which node the canvas selected, or `null` when a click cleared the selection. */
  onSelect: (nodeId: Nullable<string>) => void;
}

/**
 * One dialog as a layered graph.
 */
export function DialogGraph({ dialog, onSelect }: IDialogGraphProps): ReactElement {
  const graph: IDialogGraph = useMemo(() => buildDialogGraph(dialog), [dialog]);

  const [nodes, setNodes, onNodesChange] = useNodesState<TGraphNode>(graph.nodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState<TGraphEdge>(graph.edges);

  // Reseeded on every rebuild. Without this the hooks keep their first graph and switching dialogs
  // draws the previous one, because `useNodesState` treats its argument as an initial value.
  useEffect(() => {
    setNodes(graph.nodes);
    setEdges(graph.edges);
  }, [graph, setEdges, setNodes]);

  // Reported from the canvas rather than tracked beside it: selection is the canvas's own state, and a
  // second copy would drift the moment a click, a marquee and a keyboard move disagree.
  const onSelectionChange = useCallback(
    ({ nodes: selected }: OnSelectionChangeParams) => onSelect(selected[0]?.id ?? null),
    [onSelect]
  );

  return (
    <GraphCanvas
      data-testid={"dialog-graph"}
      nodes={nodes}
      edges={edges}
      nodeTypes={NODE_TYPES}
      isConnectable={false}
      onSelectionChange={onSelectionChange}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
    />
  );
}
