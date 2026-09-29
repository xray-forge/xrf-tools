import { Node } from "three/webgpu";

/**
 * One streak through its current fall: where it is, which way it goes, how long it has fallen and when it lands.
 */
export interface IRainFall {
  /** Where its leading end is, in renderer space. */
  head: Node<"vec3">;
  direction: Node<"vec3">;
  /** Seconds it has been falling this time. */
  age: Node<"float">;
  /** Seconds after it started that it lands, past its fall where it lands on nothing. */
  landing: Node<"float">;
  /** Where it lands. */
  landed: Node<"vec3">;
  /** Whether it is one the rain draws at all. */
  isFalling: Node<"bool">;
  /** Draws from nought to one for this fall, for what else differs between falls. */
  random: (draw: number) => Node<"float">;
}
