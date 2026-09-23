import { attribute, float, Fn } from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { isListedBuild, toListedHemiTerms } from "#/shader/placement.tsl";
import { EVertexAttribute } from "#/shader/vertex-attribute";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * `position.w` of the deferred vertex shaders: the hemisphere term the normal's fourth byte carries, scaled and offset
 * per instance for a tree (`I.Nh.w * c_scale.w + c_bias.w`), and one for a geometry that carries none. An instanced
 * static draw reads its terms from the place listed for its instance.
 *
 * @param buffers - What static draws are placed by.
 * @returns The term.
 */
export function toVertexHemi(buffers: StaticDrawBuffers): Node<"float"> {
  return Fn((_: [], builder: NodeBuilder): Node<"float"> => {
    if (!builder.geometry?.hasAttribute(EVertexAttribute.HEMI)) {
      return float(1);
    }

    const hemi: Node<"float"> = attribute<"float">(EVertexAttribute.HEMI, "float");

    if (isListedBuild(builder)) {
      const terms: Node<"vec2"> = toListedHemiTerms(buffers);

      return hemi.mul(terms.x).add(terms.y);
    }

    if (!builder.geometry.hasAttribute(EVertexAttribute.INSTANCE_HEMI)) {
      return hemi;
    }

    const terms: Node<"vec2"> = attribute<"vec2">(EVertexAttribute.INSTANCE_HEMI, "vec2");

    return hemi.mul(terms.x).add(terms.y);
  })();
}
