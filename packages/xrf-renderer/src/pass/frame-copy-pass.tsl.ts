import { texture } from "three/tsl";
import { Node, Texture } from "three/webgpu";

/**
 * @param frame - A frame.
 * @returns It, as it is.
 */
export function toFrameCopy(frame: Texture): Node<"vec4"> {
  return texture(frame);
}
