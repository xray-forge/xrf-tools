import { RenderGraphSettings } from "@/core/ipc/types/xrf-renderer";

/** Every mechanism of the frame graph on: what frames compile with unless one is turned off to bisect a capture. */
export const DEFAULT_RENDER_GRAPH_SETTINGS: Readonly<RenderGraphSettings> = {
  isCulling: true,
  isGrouping: true,
  isMerging: true,
  isPooling: true,
};

/**
 * @param graph - What the frames compile with.
 * @returns Whether any mechanism is off, so the frames do not draw as they normally do.
 */
export function isRenderGraphBisected(graph: RenderGraphSettings): boolean {
  return !graph.isCulling || !graph.isGrouping || !graph.isMerging || !graph.isPooling;
}
