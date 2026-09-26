import { fract } from "three/tsl";
import { Node } from "three/webgpu";

/**
 * `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine, which both the trees
 * and the grass sway by.
 *
 * @param phase - How far round, in turns.
 * @returns The wave.
 */
export function toCyclic(phase: Node<"float">): Node<"float"> {
  const f: Node<"float"> = fract(phase).mul(2.8284271).sub(1.4142136);

  return f.mul(f).sub(1);
}
