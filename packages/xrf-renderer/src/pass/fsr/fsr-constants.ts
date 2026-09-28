import { Node } from "three/webgpu";

/** `cbFSR2`'s fields the passes read, as uniforms. */
export interface IFsrConstants {
  renderSize: Node<"vec2">;
  displaySize: Node<"vec2">;
  /** In FSR's sense: a drawn texel `m` stands at `m + 0.5 - jitter`. */
  jitter: Node<"vec2">;
  /** The drawing's size over the display's. */
  downscale: Node<"vec2">;
  /** `fDeviceToViewDepth`: device depth to view depth, `[1] / (d - [0])`, and the projection's inverse scales. */
  deviceToView: Node<"vec4">;
  lumaMipSize: Node<"vec2">;
  jitterPhaseCount: Node<"float">;
  /** Zero on the first frame after a reset. */
  frameIndex: Node<"float">;
}
