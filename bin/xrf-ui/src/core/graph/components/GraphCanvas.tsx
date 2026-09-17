import { useColorScheme } from "@mui/material/styles";
import {
  Background,
  ColorMode,
  Controls,
  EdgeTypes,
  NodeTypes,
  OnConnect,
  OnEdgesChange,
  OnNodesChange,
  OnSelectionChangeFunc,
  ProOptions,
  ReactFlow,
  SelectionMode,
} from "@xyflow/react";
import { ReactElement, ReactNode } from "react";

import { TGraphEdge, TGraphNode } from "@/core/graph/lib/graph.types";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import "@xyflow/react/dist/style.css";

const PRO_OPTIONS: ProOptions = { hideAttribution: true };

// Middle button only. Left drag draws a selection box, and right stays free for a context menu,
// which is how every node editor in this domain expects to be driven.
const PAN_MOUSE_BUTTONS: Array<number> = [1];

interface IGraphCanvasProps extends BaseComponentProps {
  nodes: Array<TGraphNode>;
  edges: Array<TGraphEdge>;
  nodeTypes?: NodeTypes;
  edgeTypes?: EdgeTypes;
  /** Whether handles accept new links. A read-only surface still draws them, and refuses. */
  isConnectable?: boolean;
  /** Frames the graph on first render. Off for a surface restoring a saved viewport. */
  isFitToView?: boolean;
  /** Overlays inside the viewport, such as a minimap or a legend panel. */
  children?: ReactNode;
  onNodesChange?: OnNodesChange;
  onEdgesChange?: OnEdgesChange;
  onConnect?: OnConnect;
  /** What is selected now, reported rather than controlled. */
  onSelectionChange?: OnSelectionChangeFunc;
}

/**
 * The node canvas every graph surface draws on.
 *
 * The wrapper carries `minWidth`/`minHeight` of zero because a flex item defaults to `min-width:
 * auto` and refuses to shrink below its content, which leaves the canvas drawing under whatever sits
 * beside it.
 */
export function GraphCanvas({
  "data-testid": dataTestId = "graph-canvas",
  id,
  className,
  nodes,
  edges,
  nodeTypes,
  edgeTypes,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onSelectionChange,
  isConnectable = true,
  isFitToView = true,
  children,
}: IGraphCanvasProps): ReactElement {
  const { mode } = useColorScheme();

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex h-full min-h-0 w-full min-w-0 grow", className)}>
      <ReactFlow
        colorMode={(mode ?? "system") as ColorMode}
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        nodesConnectable={isConnectable}
        proOptions={PRO_OPTIONS}
        fitView={isFitToView}
        selectionOnDrag={true}
        selectionMode={SelectionMode.Partial}
        panOnDrag={PAN_MOUSE_BUTTONS}
        elevateEdgesOnSelect={true}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onSelectionChange={onSelectionChange}
      >
        <Background gap={12} size={1} />
        <Controls position={"bottom-right"} />
        {children}
      </ReactFlow>
    </div>
  );
}
