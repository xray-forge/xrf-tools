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

/** What an observer reads of a render object: the bundle group recording it, if any, and the camera it draws with. */
export interface IObservedObject {
  bundle: Nullable<{ version: number }>;
  camera?: unknown;
  /** Its render context, which three gives the camera of every render call. */
  context?: { camera?: unknown };
}

/** What an observer reads of the frame: which render call it is, and the camera its uniforms are read from. */
export interface IObservedFrame {
  renderId: number;
  camera?: unknown;
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

/**
 * Points a render object, and the frame refreshing it, at the camera its render call draws with. Three keys render
 * objects by object, material and render context but not camera, and sets a kept one's camera only when it records: a
 * bundle replayed under another camera into the same target, as every light face is, would read the camera it last
 * recorded with.
 *
 * @param renderObject - The render object about to draw.
 * @param nodeFrame - The frame refreshing it.
 */
export function adoptRenderCamera(renderObject: IObservedObject, nodeFrame: IObservedFrame): void {
  const camera: unknown = renderObject.context?.camera;

  if (camera) {
    renderObject.camera = camera;
    nodeFrame.camera = camera;
  }
}
