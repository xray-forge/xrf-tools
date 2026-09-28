import * as THREE from "three/webgpu";

/** One of three's `RenderObjectRefreshType` values. */
export type TRenderObjectRefreshType = number;

/** Three's refresh types by name. */
type TRenderObjectRefreshTypes = Readonly<Record<"NONE" | "SHARED" | "FULL", TRenderObjectRefreshType>>;

/** What three does for a render object before drawing it: its `RenderObjectRefreshType`, which its types leave out. */
export const RenderObjectRefreshType: TRenderObjectRefreshTypes = (
  THREE as unknown as { RenderObjectRefreshType: TRenderObjectRefreshTypes }
).RenderObjectRefreshType;
