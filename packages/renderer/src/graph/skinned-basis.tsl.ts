import { attribute, Fn, reference, referenceBuffer, tangentLocal, vec4 } from "three/tsl";
import { Node, NodeBuilder, SkinnedMesh } from "three/webgpu";

/** A buffer of matrices a node indexes, which the typings leave off `referenceBuffer`. */
interface IMatrixBuffer {
  element(index: Node<"uint">): Node<"mat4">;
}

/**
 * The authored tangent, skinned: three skins a geometry's `tangent` with its normal.
 */
export const skinnedTangent: Node<"vec3"> = tangentLocal;

/**
 * The authored binormal, skinned by the same bone matrices as the tangent, as the engine skins both.
 * Three skins position, normal and tangent only; this repeats its blend for the binormal, per object.
 */
export const skinnedBinormal = Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
  const binormal: Node<"vec3"> = attribute<"vec3">("binormal", "vec3");
  const object = builder.object as Partial<SkinnedMesh>;

  if (!object.isSkinnedMesh || !object.skeleton) {
    return binormal;
  }

  // Unbound, so each resolves against the object drawn, as three's own skinning does.
  const bones = referenceBuffer(
    "skeleton.boneMatrices",
    "mat4",
    object.skeleton.bones.length,
    null
  ) as unknown as IMatrixBuffer;
  const index = attribute<"uvec4">("skinIndex", "uvec4");
  const weight = attribute<"vec4">("skinWeight", "vec4");
  const skin: Node<"mat4"> = bones
    .element(index.x)
    .mul(weight.x)
    .add(bones.element(index.y).mul(weight.y))
    .add(bones.element(index.z).mul(weight.z))
    .add(bones.element(index.w).mul(weight.w));
  const bind = reference("bindMatrix", "mat4", null) as unknown as Node<"mat4">;
  const bindInverse = reference("bindMatrixInverse", "mat4", null) as unknown as Node<"mat4">;

  return bindInverse.mul(skin).mul(bind).mul(vec4(binormal, 0)).xyz;
});
