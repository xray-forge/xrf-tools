import { DialogDescriptor } from "@xrf/ipc/types/xrf-dialog";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { GraphCanvas, TGraphEdge, TGraphNode } from "@xrf/ui/graph";
import { NodeTypes, OnSelectionChangeParams, useEdgesState, useNodesState } from "@xyflow/react";
import { ReactElement, useCallback, useEffect, useEffectEvent, useMemo } from "react";

import { DialogGraphNode, DialogPhraseGraphNode } from "@/tools/dialogs-editor/components/editor/DialogGraphNode";
import { buildDialogGraph, EDialogGraphNodeType, IDialogGraph } from "@/tools/dialogs-editor/lib";

const NODE_TYPES: NodeTypes = {
  [EDialogGraphNodeType.DIALOG]: DialogGraphNode,
  [EDialogGraphNodeType.PHRASE]: DialogPhraseGraphNode,
};

/** `nodes` with `nodeId` selected, since a canvas seeded with none reports its selection cleared. */
function toSeededNodes(nodes: Array<TGraphNode>, nodeId: Nullable<string>): Array<TGraphNode> {
  return nodes.map((node: TGraphNode) => (node.id === nodeId ? { ...node, selected: true } : node));
}

export interface IDialogGraphProps extends BaseComponentProps {
  dialog: DialogDescriptor;
  /** Which node is inspected: the canvas starts with it selected, and keeps it across a rebuild of the same dialog. */
  selectedNodeId: Nullable<string>;
  /** Which node the canvas selected, or `null` when a click cleared the selection. */
  onSelect: (nodeId: Nullable<string>) => void;
}

/**
 * One dialog as a layered graph.
 */
export function DialogGraph({
  "data-testid": dataTestId = "dialog-graph",
  id,
  className,
  dialog,
  selectedNodeId,
  onSelect,
}: IDialogGraphProps): ReactElement {
  const graph: IDialogGraph = useMemo(() => buildDialogGraph(dialog), [dialog]);

  const [nodes, setNodes, onNodesChange] = useNodesState<TGraphNode>(toSeededNodes(graph.nodes, selectedNodeId));
  const [edges, setEdges, onEdgesChange] = useEdgesState<TGraphEdge>(graph.edges);

  // Read at the rebuild rather than reseeding on every click, which would put back nodes the reader dragged.
  const onRebuild = useEffectEvent((rebuilt: IDialogGraph) => {
    setNodes(toSeededNodes(rebuilt.nodes, selectedNodeId));
    setEdges(rebuilt.edges);
  });

  // Reported from the canvas rather than tracked beside it: selection is the canvas's own state, and a
  // second copy would drift the moment a click, a marquee and a keyboard move disagree.
  const onSelectionChange = useCallback(
    ({ nodes: selected }: OnSelectionChangeParams) => onSelect(selected[0]?.id ?? null),
    [onSelect]
  );

  // Reseeded on every rebuild. Without this the hooks keep their first graph and switching dialogs
  // draws the previous one, because `useNodesState` treats its argument as an initial value.
  useEffect(() => onRebuild(graph), [graph]);

  return (
    <GraphCanvas
      data-testid={dataTestId}
      id={id}
      className={className}
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
