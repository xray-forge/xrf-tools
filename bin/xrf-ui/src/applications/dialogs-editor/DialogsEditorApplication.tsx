import { addEdge, Connection, Edge, NodeTypes, useEdgesState, useNodesState } from "@xyflow/react";
import { ReactElement, useCallback } from "react";

import { DialogNode } from "@/applications/dialogs-editor/components/DialogNode";
import { PhraseNode } from "@/applications/dialogs-editor/components/PhraseNode";
import { EGraphNodeType } from "@/applications/dialogs-editor/types";
import { GraphCanvas } from "@/core/graph/components/GraphCanvas";
import { layoutGraphNodes, TGraphEdge, TGraphNode } from "@/core/graph/lib";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";

const NODE_TYPES: NodeTypes = {
  [EGraphNodeType.DIALOG_NODE]: DialogNode,
  [EGraphNodeType.PHRASE_NODE]: PhraseNode,
};

const SAMPLE_EDGES: Array<TGraphEdge> = [
  { id: "dialog-0", source: "dialog", target: "0" },
  { id: "0-1", source: "0", target: "1" },
  { id: "1-2", source: "1", target: "2" },
];

// Positions come from the layout rather than from literals, so the sample proves the shared canvas
// and its layouter work together. Real dialogs replace all of this in the read-only phase.
const SAMPLE_NODES: Array<TGraphNode> = layoutGraphNodes(
  [
    {
      data: { label: "zat_b30_owl_stalker_trader", tags: ["precondition", "dont_has_info"] },
      id: "dialog",
      position: { x: 0, y: 0 },
      type: EGraphNodeType.DIALOG_NODE,
    },
    {
      data: { label: "Owl, I hear you buy unusual things.", tags: ["text"] },
      id: "0",
      position: { x: 0, y: 0 },
      type: EGraphNodeType.PHRASE_NODE,
    },
    {
      data: { label: "Depends what you are carrying.", tags: ["text", "precondition"] },
      id: "1",
      position: { x: 0, y: 0 },
      type: EGraphNodeType.PHRASE_NODE,
    },
    {
      data: { label: "Then we have nothing to discuss.", tags: ["text", "give_info", "is_final"] },
      id: "2",
      position: { x: 0, y: 0 },
      type: EGraphNodeType.PHRASE_NODE,
    },
  ],
  SAMPLE_EDGES
);

/**
 * Signpost for the dialog graph editor.
 *
 * A sample graph on the shared canvas, with no project, no backend and no persistence. The
 * application stays `PLANNED` until it can open real dialog XML.
 */
export function DialogsEditorApplication(): ReactElement {
  const [nodes, , onNodesChange] = useNodesState(SAMPLE_NODES);
  const [edges, setEdges, onEdgesChange] = useEdgesState(SAMPLE_EDGES);

  const onConnect = useCallback(
    (connection: Connection) => setEdges((current: Array<Edge>) => addEdge(connection, current)),
    [setEdges]
  );

  return (
    <EditorLayout toolbar={<EditorToolbar />}>
      <GraphCanvas
        nodes={nodes}
        edges={edges}
        nodeTypes={NODE_TYPES}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
      />
    </EditorLayout>
  );
}
