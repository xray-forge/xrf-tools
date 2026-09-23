import { Nullable } from "@xrf/types";
import * as THREE from "three/webgpu";
import { NodeMaterialObserver } from "three/webgpu";

/** One of three's `RenderObjectRefreshType` values. */
export type TRenderObjectRefreshType = number;

/** Three's refresh types by name. */
type TRenderObjectRefreshTypes = Readonly<Record<"NONE" | "SHARED" | "FULL", TRenderObjectRefreshType>>;

/** What three does for a render object before drawing it: its `RenderObjectRefreshType`, which its types leave out. */
export const RenderObjectRefreshType: TRenderObjectRefreshTypes = (
  THREE as unknown as { RenderObjectRefreshType: TRenderObjectRefreshTypes }
).RenderObjectRefreshType;

/** What an observer reads of a render object: the bundle group recording it, if any. */
export interface IObservedObject {
  bundle: Nullable<{ version: number }>;
}

/** What an observer reads of the frame: which render call it is. */
export interface IObservedFrame {
  renderId: number;
}

/** Three's own refresh decision, which its types leave out. */
type TNeedsRefresh = (renderObject: IObservedObject, nodeFrame: IObservedFrame) => TRenderObjectRefreshType;

/**
 * @param observer - The observer deciding, as three's own observer would.
 * @param renderObject - The render object about to draw.
 * @param nodeFrame - The frame it draws in.
 * @returns What three's own observer would refresh.
 */
export function callBaseNeedsRefresh(
  observer: NodeMaterialObserver,
  renderObject: IObservedObject,
  nodeFrame: IObservedFrame
): TRenderObjectRefreshType {
  return (NodeMaterialObserver.prototype as unknown as { needsRefresh: TNeedsRefresh }).needsRefresh.call(
    observer,
    renderObject,
    nodeFrame
  );
}
