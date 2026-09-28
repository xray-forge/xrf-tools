import { Fn, If, uint } from "three/tsl";
import { Node, UniformArrayNode } from "three/webgpu";

/**
 * @param sphere - A sphere in renderer space, a negative radius for one standing for nothing.
 * @param planes - The view's six planes, normals pointing in, `w` the constant.
 * @returns One where the sphere reaches into the view, zero where it does not or stands for nothing.
 */
export function toInFrustum(sphere: Node<"vec4">, planes: UniformArrayNode<string>): Node<"uint"> {
  return Fn(() => {
    const isSeen = uint(1).toVar();

    If(sphere.w.lessThan(0), () => {
      isSeen.assign(0);
    });

    for (let plane = 0; plane < 6; plane += 1) {
      const it = planes.element(plane) as unknown as Node<"vec4">;

      If(it.xyz.dot(sphere.xyz).add(it.w).lessThan(sphere.w.negate()), () => {
        isSeen.assign(0);
      });
    }

    return isSeen;
  })();
}
