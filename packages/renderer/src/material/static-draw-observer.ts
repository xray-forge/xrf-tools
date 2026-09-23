import { Nullable } from "@xrf/types";
import { NodeMaterialObserver } from "three/webgpu";

/** What three does for a render object before drawing it: its `RenderObjectRefreshType`, which its types leave out. */
enum ERefreshType {
  NONE = 0,
  SHARED = 1,
  FULL = 2,
}

/** What the observer reads of a render object: the bundle recording it, if any. */
interface IObservedObject {
  bundle: Nullable<{ version: number }>;
}

/** What the observer reads of the frame: which render call it is. */
interface IObservedFrame {
  renderId: number;
}

/** Three's own refresh decision, which its types leave out. */
type TNeedsRefresh = (renderObject: IObservedObject, nodeFrame: IObservedFrame) => ERefreshType;

/**
 * How three refreshes a static draw's render object before drawing it: fully when its bundle records, and on a replay
 * only the uniform groups every material shares. Three refreshes every node material fully on every replay, since a
 * node could read anything; a static draw's shader reads nothing of its own object that changes without its bundle
 * recording again - its textures, arena and slot changes all record it - so that refresh bought nothing and cost
 * every batch several microseconds a frame.
 */
export class StaticDrawObserver extends NodeMaterialObserver {
  /** The bundle version each render object was last fully refreshed at. */
  private readonly recorded: WeakMap<IObservedObject, number> = new WeakMap();

  public needsRefresh(renderObject: IObservedObject, nodeFrame: IObservedFrame): ERefreshType {
    const { bundle } = renderObject;

    if (!bundle || this.recorded.get(renderObject) !== bundle.version) {
      if (bundle) {
        this.recorded.set(renderObject, bundle.version);
      }

      return (NodeMaterialObserver.prototype as unknown as { needsRefresh: TNeedsRefresh }).needsRefresh.call(
        this,
        renderObject,
        nodeFrame
      );
    }

    // Shared groups are one buffer for every object using them: refreshed once a render call.
    if (this.renderId !== nodeFrame.renderId) {
      this.renderId = nodeFrame.renderId;

      return ERefreshType.SHARED;
    }

    return ERefreshType.NONE;
  }
}
