import { PerspectiveCamera } from "three/webgpu";

import { IRendererLightsSettings } from "#/contract/renderer-lights-settings";
import { LodUniforms } from "#/uniforms/lod-uniforms";

/** What a frame's lights are written for. */
export interface ISceneLightsFrame {
  /** The view's camera, unjittered: what the lights are culled, ordered, faded and sized by. */
  readonly view: PerspectiveCamera;
  /** The camera the scene draws with, whose projection the clusters cut. */
  readonly camera: PerspectiveCamera;
  /** Seconds, which the animations run by. */
  readonly time: number;
  readonly settings: IRendererLightsSettings;
  /** The level of detail thresholds, which shadowed lights fade by. */
  readonly lod: LodUniforms;
  /** Whether the wind sways the trees this frame, which has the faces over them drawn again. */
  readonly isWindy: boolean;
}
