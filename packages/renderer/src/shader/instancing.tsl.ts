import {
  attribute,
  Fn,
  mat3,
  mat4,
  modelViewMatrix,
  normalize,
  normalLocal,
  normalView,
  positionLocal,
  varying,
  vec3,
  vec4,
} from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { INSTANCE_MATRIX_COLUMNS } from "#/shader/vertex-attribute";

/** Whether the geometry being built for stands in many places through instanced attributes. */
function isInstancedBuild(builder: NodeBuilder): boolean {
  return Boolean(builder.geometry?.hasAttribute(INSTANCE_MATRIX_COLUMNS[0]));
}

/** A place's transform, from the four columns its instanced attributes carry. */
function toInstanceMatrix(): Node<"mat4"> {
  const [x, y, z, w] = INSTANCE_MATRIX_COLUMNS.map((column: string) => attribute<"vec4">(column, "vec4"));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/**
 * The local position, stood in its place for a geometry drawn in many: instanced by vertex attributes rather than by
 * three's `InstancedMesh`, whose shader carries its instance count, so every stand of trees was a pipeline of its own.
 */
export const instancedPosition = Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
  if (!isInstancedBuild(builder)) {
    return positionLocal;
  }

  return toInstanceMatrix().mul(vec4(positionLocal, 1)).xyz;
});

/**
 * The view normal, turned by the place's transform where there is one, as three's own instancing turns it: divided by
 * the squared scale of each axis first, so a stretched place does not bend its normals.
 */
export const instancedNormalView = Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
  if (!isInstancedBuild(builder)) {
    return normalView;
  }

  const m = mat3(toInstanceMatrix() as unknown as Node<"mat3">) as unknown as Node<"mat3"> &
    ReadonlyArray<Node<"vec3">>;
  const scaled: Node<"vec3"> = normalLocal.div(vec3(m[0].dot(m[0]), m[1].dot(m[1]), m[2].dot(m[2])));

  return normalize(varying(modelViewMatrix.mul(vec4(m.mul(scaled), 0)).xyz));
});
