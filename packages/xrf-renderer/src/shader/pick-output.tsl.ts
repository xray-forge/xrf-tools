import { float, Fn, instanceIndex, positionView, uniform, uvec2, varying, vec4 } from "three/tsl";
import { Node, NodeBuilder, NodeFrame } from "three/webgpu";

import { IClusterSource, toClusterSource } from "#/geometry/cluster-source";
import { isClusteredBuild, toClusterEntry } from "#/shader/cluster-vertex.tsl";
import { EPickKind } from "#/shader/pick-kind";

/**
 * What a pick writes where a surface stands: what kind of draw it is, which draw and which place of it, and how far
 * from the eye. A static draw names its slot, read from its cluster, and its place; a plain one its mesh and the
 * instance of it drawn. Written as floats, each exact under 2^24.
 *
 * @returns The texel.
 */
export function toPickOutput(): Node<"vec4"> {
  return Fn((_: [], builder: NodeBuilder): Node<"vec4"> => {
    const distance: Node<"float"> = positionView.length();

    if (isClusteredBuild(builder)) {
      const source: IClusterSource = toClusterSource(builder.geometry) as IClusterSource;
      const entry: Node<"uvec2"> = toClusterEntry(builder);
      const range = source.rangeNode.element(entry.x) as unknown as Node<"uvec4">;
      // Integers cross to the fragment stage flat.
      const drawn = varying(uvec2(range.w, entry.y)) as unknown as Node<"uvec2">;

      return vec4(float(EPickKind.STATIC), float(drawn.x), float(drawn.y), distance) as unknown as Node<"vec4">;
    }

    const mesh: Node<"float"> = uniform(0).onObjectUpdate(({ object }: NodeFrame) => object?.id ?? 0);
    const instance = varying(instanceIndex) as unknown as Node<"uint">;

    return vec4(float(EPickKind.PLAIN), mesh, float(instance), distance) as unknown as Node<"vec4">;
  })() as unknown as Node<"vec4">;
}
