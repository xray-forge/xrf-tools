import { attribute, Fn, positionGeometry, reference, referenceBuffer, tangentLocal, vec4 } from "three/tsl";
import { Node, NodeBuilder, SkinnedMesh } from "three/webgpu";

import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { PREVIOUS_BONE_MATRICES } from "#/shader/previous-skeleton";

/** A buffer of matrices a node indexes, which the typings leave off `referenceBuffer`. */
interface IMatrixBuffer {
  element(index: Node<"uint">): Node<"mat4">;
}

/**
 * A vertex's skin, as three's `SkinningNode` blends it: its four bones' matrices weighted, between the bind matrices.
 *
 * @param bones - The bone matrices, current or the frame before's.
 * @returns What turns a model space vector of the vertex's.
 */
function toSkinMatrix(bones: IMatrixBuffer): Node<"mat4"> {
  const index: Node<"uvec4"> = attribute<"uvec4">("skinIndex", "uvec4");
  const weight: Node<"vec4"> = attribute<"vec4">("skinWeight", "vec4");
  const skin: Node<"mat4"> = bones
    .element(index.x)
    .mul(weight.x)
    .add(bones.element(index.y).mul(weight.y))
    .add(bones.element(index.z).mul(weight.z))
    .add(bones.element(index.w).mul(weight.w));
  const bind: Node<"mat4"> = reference("bindMatrix", "mat4", null) as unknown as Node<"mat4">;
  const bindInverse: Node<"mat4"> = reference("bindMatrixInverse", "mat4", null) as unknown as Node<"mat4">;

  return bindInverse.mul(skin).mul(bind);
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
  const binormal: Node<"vec3"> = attribute<"vec3">(EVertexAttribute.BINORMAL, "vec3");
  const object: Partial<SkinnedMesh> = builder.object as Partial<SkinnedMesh>;

  if (!object.isSkinnedMesh || !object.skeleton) {
    return binormal;
  }

  // Unbound, so each resolves against the object drawn, as three's own skinning does.
  const bones: IMatrixBuffer = referenceBuffer(
    "skeleton.boneMatrices",
    "mat4",
    object.skeleton.bones.length,
    null
  ) as unknown as IMatrixBuffer;

  return toSkinMatrix(bones).mul(vec4(binormal, 0)).xyz;
});

/**
 * Where a skinned vertex stood the frame before: its geometry's position blended by the bone matrices its skeleton kept
 * from that frame (`previousBoneMatrices`), as three blends the current ones.
 */
export const previousSkinnedPosition = Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
  const object: Partial<SkinnedMesh> = builder.object as Partial<SkinnedMesh>;

  if (!object.isSkinnedMesh || !object.skeleton) {
    return positionGeometry;
  }

  const bones: IMatrixBuffer = referenceBuffer(
    `skeleton.${PREVIOUS_BONE_MATRICES}`,
    "mat4",
    object.skeleton.bones.length,
    null
  ) as unknown as IMatrixBuffer;

  return toSkinMatrix(bones).mul(vec4(positionGeometry, 1)).xyz;
});
