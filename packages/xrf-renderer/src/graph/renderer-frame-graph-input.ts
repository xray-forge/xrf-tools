import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { RendererScene } from "#/scene/renderer-scene";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** What the frame is drawn from. */
export interface IRendererFrameGraphInput {
  /** What the frame's shaders read. */
  uniforms: RendererUniforms;
  /** The helpers drawn last. */
  overlays: RendererOverlays;
  /** What the consumer put that the frame's own passes draw: the static cull and casters, grass, lights and rain. */
  scene: Pick<RendererScene, "staticCull" | "shadowCasters" | "grass" | "lights" | "rain">;
}
