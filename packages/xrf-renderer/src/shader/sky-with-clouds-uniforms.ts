import { Node } from "three/webgpu";

import { CloudUniforms } from "#/uniforms/cloud-uniforms";
import { EngineUniforms } from "#/uniforms/engine-uniforms";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

/** What the sky is drawn with, the clouds laid over it. */
export interface ISkyWithCloudsUniforms {
  sky: SkyUniforms;
  clouds: CloudUniforms;
  /** The engine the sky is drawn as. */
  engine: EngineUniforms;
  /** What the tonemap multiplies by. */
  scale: Node<"float">;
}
