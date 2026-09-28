import { Node } from "three/webgpu";

/** The world slot a cell of the ring holds while the camera stands where it does, and how many rings out it is. */
export interface IGrassRingSlot {
  x: Node<"int">;
  z: Node<"int">;
  /** Slots from the camera's along the farther axis: nought for the camera's own. */
  band: Node<"int">;
}
