import { Texture } from "three/webgpu";

import { EngineUniforms } from "#/uniforms/engine-uniforms";
import { WetUniforms } from "#/uniforms/wet-uniforms";

/** What `rain_apply_gloss` reads. */
export interface IWetGlossInput {
  /** What the patch wrote: the normal, and the wetness in alpha. */
  patched: Texture;
  /** The G-buffer's depth. */
  depth: Texture;
  wet: WetUniforms;
  engine: EngineUniforms;
}
