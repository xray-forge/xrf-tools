import {
  attribute,
  cameraViewMatrix,
  Fn,
  mat3,
  mat4,
  modelViewMatrix,
  normalize,
  normalLocal,
  normalView,
  positionLocal,
  storage,
  varying,
  vec3,
  vec4,
} from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { EVertexAttribute, INSTANCE_MATRIX_COLUMNS } from "#/shader/vertex-attribute";
import { STATIC_DRAW_CAPACITY, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

// Where a vertex stands: in each place its instanced attributes name, where the static draw buffers put it, or where
// its object's matrix puts it. The first two read no uniform of the object's own, so three refreshes nothing per
// object for them; which one a shader is built for follows from its geometry's attributes, as three's shader cache
// does.

/** Whether the geometry being built for stands in many places through instanced attributes. */
function isInstancedBuild(builder: NodeBuilder): boolean {
  return Boolean(builder.geometry?.hasAttribute(INSTANCE_MATRIX_COLUMNS[0]));
}

/**
 * @param builder - The builder of the shader in question.
 * @returns Whether the geometry it builds for is a static draw, placed by the buffers every static draw shares.
 */
export function isStaticBuild(builder: NodeBuilder): boolean {
  return Boolean(builder.geometry?.hasAttribute(EVertexAttribute.STATIC_SLOT));
}

/** A place's transform, from the four columns its instanced attributes carry. */
function toInstanceMatrix(): Node<"mat4"> {
  const [x, y, z, w] = INSTANCE_MATRIX_COLUMNS.map((column: string) => attribute<"vec4">(column, "vec4"));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/** A static draw's matrix, from its slot in the shared buffers. */
function toStaticMatrix(buffers: StaticDrawBuffers): Node<"mat4"> {
  const columns = storage(buffers.models, "vec4", STATIC_DRAW_CAPACITY * 4);
  const first: Node<"uint"> = attribute<"uint">(EVertexAttribute.STATIC_SLOT, "uint").mul(4);
  const [x, y, z, w] = [0, 1, 2, 3].map((column: number) => columns.element(first.add(column)));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/** A transform's normals, divided by the squared scale of each axis first, so a stretched place does not bend them. */
function toTransformedNormal(matrix: Node<"mat4">): Node<"vec3"> {
  const m = mat3(matrix as unknown as Node<"mat3">) as unknown as Node<"mat3"> & ReadonlyArray<Node<"vec3">>;

  return m.mul(normalLocal.div(vec3(m[0].dot(m[0]), m[1].dot(m[1]), m[2].dot(m[2]))));
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
 * @param buffers - What static draws are placed by.
 * @returns A static draw's position in view space: its matrix, then the camera's, and nothing of its object's.
 */
export function toStaticPositionView(buffers: StaticDrawBuffers): Node<"vec3"> {
  return cameraViewMatrix.mul(toStaticMatrix(buffers).mul(vec4(positionLocal, 1))).xyz;
}

/**
 * @param buffers - What static draws are placed by.
 * @returns The view normal, turned by whatever places the vertex, as three's own instancing turns it.
 */
export function toPlacedNormalView(buffers: StaticDrawBuffers): Node<"vec3"> {
  return Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
    if (isStaticBuild(builder)) {
      return normalize(varying(cameraViewMatrix.mul(vec4(toTransformedNormal(toStaticMatrix(buffers)), 0)).xyz));
    }

    if (isInstancedBuild(builder)) {
      return normalize(varying(modelViewMatrix.mul(vec4(toTransformedNormal(toInstanceMatrix()), 0)).xyz));
    }

    return normalView;
  })();
}

/**
 * @param direction - A direction in the geometry's own space, such as its authored tangent.
 * @param buffers - What static draws are placed by.
 * @returns The direction in view space, turned by whatever places the vertex.
 */
export function toPlacedViewDirection(direction: Node<"vec3">, buffers: StaticDrawBuffers): Node<"vec3"> {
  return Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
    if (isStaticBuild(builder)) {
      return cameraViewMatrix.mul(toStaticMatrix(buffers).mul(vec4(direction, 0))).xyz;
    }

    return modelViewMatrix.mul(vec4(direction, 0)).xyz;
  })();
}
