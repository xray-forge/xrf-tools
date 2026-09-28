import { Camera, RenderTarget } from "three/webgpu";

/** Where the shadow materials compile: a cascade's target, and its camera, as the shadow passes draw them. */
export interface IFrameShadowCompile {
  readonly target: RenderTarget;
  readonly camera: Camera;
}
