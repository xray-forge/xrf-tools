import { Node } from "three/webgpu";

import { CloudUniforms } from "#/uniforms/cloud-uniforms";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

/** What the sky is drawn with, the clouds laid over it. */
export interface ISkyWithCloudsUniforms {
  sky: SkyUniforms;
  clouds: CloudUniforms;
  /** What the tonemap multiplies by. */
  scale: Node<"float">;
}
