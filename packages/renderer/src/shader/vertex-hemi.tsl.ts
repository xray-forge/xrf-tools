import { attribute, float, Fn } from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { EVertexAttribute } from "#/shader/vertex-attribute";

/**
 * `position.w` of the deferred vertex shaders: the hemisphere term the normal's fourth byte carries, scaled and offset
 * per instance for a tree (`I.Nh.w * c_scale.w + c_bias.w`), and one for a geometry that carries none.
 */
export const vertexHemi = Fn((_: [], builder: NodeBuilder): Node<"float"> => {
  if (!builder.geometry?.hasAttribute(EVertexAttribute.HEMI)) {
    return float(1);
  }

  const hemi: Node<"float"> = attribute<"float">(EVertexAttribute.HEMI, "float");

  if (!builder.geometry.hasAttribute(EVertexAttribute.INSTANCE_HEMI)) {
    return hemi;
  }

  const terms: Node<"vec2"> = attribute<"vec2">(EVertexAttribute.INSTANCE_HEMI, "vec2");

  return hemi.mul(terms.x).add(terms.y);
});
