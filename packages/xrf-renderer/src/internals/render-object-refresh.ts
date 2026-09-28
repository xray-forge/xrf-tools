import { NodeMaterialObserver } from "three/webgpu";

import { IObservedFrame } from "#/internals/observed-frame";
import { IObservedObject } from "#/internals/observed-object";
import { TRenderObjectRefreshType } from "#/internals/render-object-refresh-type";

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
