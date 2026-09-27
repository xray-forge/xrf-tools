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
  transformNormalToView,
  varying,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, NodeBuilder } from "three/webgpu";

import { INSTANCE_MATRIX_COLUMNS } from "#/geometry/vertex-attribute";
import { isClusteredBuild, toClusterAttribute, toClusterEntry, toVertexAttribute } from "#/shader/cluster-vertex.tsl";
import { toCyclic } from "#/shader/cyclic-wave.tsl";
import { isPackedBuild, isPackedTreeBuild, toPackedNormal, toPackedTreeRigidity } from "#/shader/packed-vertex.tsl";
import { STATIC_PLACE_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { TreeWindUniforms } from "#/uniforms/tree-wind-uniforms";

// Where a vertex stands: in each place its instanced attributes name, where the static draw buffers put it - by the
// place of the entry its instance draws - or where its object's matrix puts it. The buffer placed ones read no uniform
// of the object's own, so three refreshes nothing per object for them; which one a shader is built for follows from its
// geometry, as three's shader cache does.

/** Whether the geometry being built for stands in many places through instanced attributes. */
function isInstancedBuild(builder: NodeBuilder): boolean {
  return Boolean(builder.geometry?.hasAttribute(INSTANCE_MATRIX_COLUMNS[0]));
}

/**
 * @param builder - The builder of the shader in question.
 * @returns Whether the buffers every static draw shares place what it builds for, so a replay refreshes nothing: a
 *   clustered static draw's, placed by the entry its instance draws.
 */
export function isBufferPlacedBuild(builder: NodeBuilder): boolean {
  return isClusteredBuild(builder);
}

/** A place's transform, from the four columns its instanced attributes carry. */
function toInstanceMatrix(): Node<"mat4"> {
  const [x, y, z, w] = INSTANCE_MATRIX_COLUMNS.map((column: string) => attribute<"vec4">(column, "vec4"));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/** The first column of the place of the entry a clustered build's instance draws. */
function toEntryPlace(builder: NodeBuilder): Node<"uint"> {
  return toClusterEntry(builder).y.mul(STATIC_PLACE_COLUMNS);
}

/** A clustered build's matrix, from the place of the entry its instance draws. */
function toBufferMatrix(builder: NodeBuilder, buffers: StaticDrawBuffers): Node<"mat4"> {
  const first: Node<"uint"> = toEntryPlace(builder);
  const [x, y, z, w] = [0, 1, 2, 3].map((column: number) => buffers.placeColumns.element(first.add(column)));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/**
 * @param buffers - What static draws are placed by.
 * @returns The impostor the place of the entry drawn is, which an impostor surface draws.
 */
export function toPlacedImpostor(buffers: StaticDrawBuffers): Node<"uint"> {
  return Fn((_: [], builder: NodeBuilder): Node<"uint"> =>
    (buffers.placeColumns.element(toEntryPlace(builder).add(4)) as unknown as Node<"vec4">).z.toUint()
  )();
}

/**
 * @param builder - The builder of a clustered build.
 * @param buffers - What static draws are placed by.
 * @returns The hemisphere scale and offset of the place of the entry drawn: a tree's own, one and none for any other.
 */
export function toPlacedHemiTerms(builder: NodeBuilder, buffers: StaticDrawBuffers): Node<"vec2"> {
  return buffers.placeColumns.element(toEntryPlace(builder).add(4)).xy as Node<"vec2">;
}

/** The vertex's normal in its geometry's own space: the engine's packed one, or its float one. */
function toLocalNormal(builder: NodeBuilder): Node<"vec3"> {
  if (isPackedBuild(builder)) {
    return toPackedNormal();
  }

  return isClusteredBuild(builder) ? toVertexAttribute("normal", "vec3") : normalLocal;
}

/** A transform's normals, divided by the squared scale of each axis first, so a stretched place does not bend them. */
function toTransformedNormal(matrix: Node<"mat4">, normal: Node<"vec3">): Node<"vec3"> {
  const m = mat3(matrix as unknown as Node<"mat3">) as unknown as Node<"mat3"> & ReadonlyArray<Node<"vec3">>;

  return m.mul(normal.div(vec3(m[0].dot(m[0]), m[1].dot(m[1]), m[2].dot(m[2]))));
}

/**
 * The local position, stood in its place for a geometry drawn in many: instanced by vertex attributes rather than by
 * three's `InstancedMesh`, whose shader carries its instance count, so every stand of trees was a pipeline of its own.
 */
export const instancedPosition = Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
  // A clustered build's vertex is its arena's, which it stands in its place itself (`toBufferPlacedWorlds`).
  if (isClusteredBuild(builder)) {
    return toClusterAttribute(builder, "position") as Node<"vec3">;
  }

  if (!isInstancedBuild(builder)) {
    return positionLocal;
  }

  return toInstanceMatrix().mul(vec4(positionLocal, 1)).xyz;
});

/**
 * @param builder - The builder of a buffer placed shader.
 * @param buffers - What static draws are placed by.
 * @param wind - How the trees sway, which a tree's vertices take in the world, as `deffer_tree_*.vs` moves them.
 * @returns A static draw's position in view space: its matrix, then the camera's, and nothing of its object's.
 */
export function toBufferPlacedPositionView(
  builder: NodeBuilder,
  buffers: StaticDrawBuffers,
  wind: TreeWindUniforms
): Node<"vec3"> {
  return cameraViewMatrix.mul(vec4(toBufferPlacedWorlds(builder, buffers, wind).current, 1)).xyz;
}

/** Where a static draw's vertex stands in the world this frame, and where it stood the frame before. */
export interface IBufferPlacedWorlds {
  readonly current: Node<"vec3">;
  readonly previous: Node<"vec3">;
}

/**
 * @param builder - The builder of a buffer placed shader.
 * @param buffers - What static draws are placed by.
 * @param wind - How the trees sway.
 * @returns A static draw's vertex in the world, both frames from one read of its matrix: it stands still but for a
 *   tree's sway, at each frame's wind.
 */
export function toBufferPlacedWorlds(
  builder: NodeBuilder,
  buffers: StaticDrawBuffers,
  wind: TreeWindUniforms
): IBufferPlacedWorlds {
  const matrix: Node<"mat4"> = toBufferMatrix(builder, buffers).toVar();
  const local: Node<"vec3"> = toClusterAttribute(builder, "position") as Node<"vec3">;
  const world: Node<"vec3"> = matrix.mul(vec4(local, 1)).xyz;

  if (!isPackedTreeBuild(builder)) {
    return { current: world, previous: world };
  }

  return {
    current: toSwayed(world, matrix, wind.wind, wind.wave),
    previous: toSwayed(world, matrix, wind.previousWind, wind.previousWave),
  };
}

/**
 * `deffer_tree_*.vs`: a tree's vertex moved across the ground by the wind, as far as its height over the tree's foot
 * times the wave at its place, and as much of that as its rigidity lets it.
 *
 * @param world - The vertex in the world.
 * @param matrix - What placed the tree, whose translation is its foot (`m_xform._24`).
 * @param wind - The engine's `wind`: which way the trees lean, and how far.
 * @param wave - The engine's `wave`: its direction through the level, and its phase.
 * @returns The vertex where the wind has it.
 */
function toSwayed(world: Node<"vec3">, matrix: Node<"mat4">, wind: Node<"vec3">, wave: Node<"vec4">): Node<"vec3"> {
  const foot: Node<"float"> = (matrix as unknown as ReadonlyArray<Node<"vec4">>)[3].y;
  const phase: Node<"float"> = toCyclic(wave.w.add(world.dot(wave.xyz)));
  const lean: Node<"vec2"> = vec2(wind.x, wind.z).mul(world.y.sub(foot).mul(phase)).mul(toPackedTreeRigidity());

  return world.add(vec3(lean.x, 0, lean.y));
}

/**
 * @param buffers - What static draws are placed by.
 * @returns The view normal, turned by whatever places the vertex, as three's own instancing turns it.
 */
export function toPlacedNormalView(buffers: StaticDrawBuffers): Node<"vec3"> {
  return Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
    if (isBufferPlacedBuild(builder)) {
      const matrix: Node<"mat4"> = toBufferMatrix(builder, buffers);

      return normalize(varying(cameraViewMatrix.mul(vec4(toTransformedNormal(matrix, toLocalNormal(builder)), 0)).xyz));
    }

    if (isInstancedBuild(builder)) {
      return normalize(
        varying(modelViewMatrix.mul(vec4(toTransformedNormal(toInstanceMatrix(), toLocalNormal(builder)), 0)).xyz)
      );
    }

    // Three's own view normal reads its float attribute, which a packed geometry does not carry.
    return isPackedBuild(builder) ? normalize(varying(transformNormalToView(toPackedNormal()))) : normalView;
  })();
}

/**
 * @param direction - A direction in the geometry's own space, such as its authored tangent.
 * @param buffers - What static draws are placed by.
 * @returns The direction in view space, turned by whatever places the vertex.
 */
export function toPlacedViewDirection(direction: Node<"vec3">, buffers: StaticDrawBuffers): Node<"vec3"> {
  return Fn((_: [], builder: NodeBuilder): Node<"vec3"> => {
    if (isBufferPlacedBuild(builder)) {
      return cameraViewMatrix.mul(toBufferMatrix(builder, buffers).mul(vec4(direction, 0))).xyz;
    }

    // Each place turns its instance's authored directions too, as it turns its normals.
    if (isInstancedBuild(builder)) {
      return modelViewMatrix.mul(toInstanceMatrix().mul(vec4(direction, 0))).xyz;
    }

    return modelViewMatrix.mul(vec4(direction, 0)).xyz;
  })();
}
