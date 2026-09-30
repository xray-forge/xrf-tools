import { abs, attribute, dot, float, floatBitsToUint, Fn, saturate, select, unpackHalf2x16, vec3 } from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { isClusteredBuild, toVertexAttribute } from "#/shader/cluster-vertex.tsl";
import { isPackedBuild, toPackedHemi } from "#/shader/packed-vertex.tsl";
import { toPlacedHemiCube, toPlacedHemiTerms, toPlacedNormalWorld } from "#/shader/placement.tsl";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * `position.w` of the deferred vertex shaders. A place with a hemisphere cube, a dynamic object's, takes it by the
 * vertex's world normal (`deffer_model_*.vs`). Otherwise it is the term the normal's fourth byte carries, scaled and
 * offset per instance for a tree (`I.Nh.w * c_scale.w + c_bias.w`), and one for a geometry that carries none. A static
 * draw reads its place of the entry its instance draws. A packed normal carries the byte itself.
 *
 * @param buffers - What static draws are placed by.
 * @returns The term.
 */
export function toVertexHemi(buffers: StaticDrawBuffers): Node<"float"> {
  return Fn((_: [], builder: NodeBuilder): Node<"float"> => {
    const own: Node<"float"> = toOwnHemi(builder, buffers);

    if (isClusteredBuild(builder)) {
      const cube: Node<"vec4"> = toPlacedHemiCube(builder, buffers).toVar();
      const words = floatBitsToUint(cube.xyz) as unknown as Node<"uvec3">;
      const first = unpackHalf2x16(words.x) as unknown as Node<"vec2">;
      const second = unpackHalf2x16(words.y) as unknown as Node<"vec2">;
      const third = unpackHalf2x16(words.z) as unknown as Node<"vec2">;
      const cubed: Node<"float"> = toCubeHemi(
        vec3(first.x, first.y, second.x),
        vec3(second.y, third.x, third.y),
        toPlacedNormalWorld(builder, buffers)
      );

      return select(cube.w.greaterThan(0.5), cubed, own);
    }

    if (builder.geometry?.hasAttribute(EVertexAttribute.INSTANCE_HEMI_POSITIVE)) {
      return toCubeHemi(
        attribute<"vec3">(EVertexAttribute.INSTANCE_HEMI_POSITIVE, "vec3"),
        attribute<"vec3">(EVertexAttribute.INSTANCE_HEMI_NEGATIVE, "vec3"),
        toPlacedNormalWorld(builder, buffers)
      );
    }

    return own;
  })();
}

/** The vertex's own term: its baked byte, scaled and offset for a tree's place, or one where it carries none. */
function toOwnHemi(builder: NodeBuilder, buffers: StaticDrawBuffers): Node<"float"> {
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
}

/**
 * `hemi_val = saturate(dot(Nw < 0 ? hc_neg : hc_pos, abs(Nw)))`: each face weighted by how far the normal faces its way.
 *
 * @param positive - The faces toward `+x +y +z`.
 * @param negative - The faces toward `-x -y -z`.
 * @param normal - The vertex's world normal.
 */
function toCubeHemi(positive: Node<"vec3">, negative: Node<"vec3">, normal: Node<"vec3">): Node<"float"> {
  const faces: Node<"vec3"> = vec3(
    select(normal.x.lessThan(0), negative.x, positive.x),
    select(normal.y.lessThan(0), negative.y, positive.y),
    select(normal.z.lessThan(0), negative.z, positive.z)
  );

  return saturate(dot(faces, abs(normal)));
}
