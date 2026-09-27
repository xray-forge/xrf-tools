import { attribute, float, Fn } from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { isClusteredBuild, toVertexAttribute } from "#/shader/cluster-vertex.tsl";
import { isPackedBuild, toPackedHemi } from "#/shader/packed-vertex.tsl";
import { toPlacedHemiTerms } from "#/shader/placement.tsl";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * `position.w` of the deferred vertex shaders: the hemisphere term the normal's fourth byte carries, scaled and offset
 * per instance for a tree (`I.Nh.w * c_scale.w + c_bias.w`), and one for a geometry that carries none. A static draw
 * reads its terms from the place of the entry its instance draws. A packed normal carries the byte itself.
 *
 * @param buffers - What static draws are placed by.
 * @returns The term.
 */
export function toVertexHemi(buffers: StaticDrawBuffers): Node<"float"> {
  return Fn((_: [], builder: NodeBuilder): Node<"float"> => {
    const isPacked: boolean = isPackedBuild(builder);

    if (!isPacked && !builder.geometry?.hasAttribute(EVertexAttribute.HEMI)) {
      return float(1);
    }

    const hemi: Node<"float"> = isPacked ? toPackedHemi() : toVertexAttribute<"float">(EVertexAttribute.HEMI, "float");

    if (isClusteredBuild(builder)) {
      const terms: Node<"vec2"> = toPlacedHemiTerms(builder, buffers);

      return hemi.mul(terms.x).add(terms.y);
    }

    if (!builder.geometry.hasAttribute(EVertexAttribute.INSTANCE_HEMI)) {
      return hemi;
    }

    const terms: Node<"vec2"> = attribute<"vec2">(EVertexAttribute.INSTANCE_HEMI, "vec2");

    return hemi.mul(terms.x).add(terms.y);
  })();
}
