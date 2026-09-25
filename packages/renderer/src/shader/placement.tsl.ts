import {
  attribute,
  cameraViewMatrix,
  Fn,
  fract,
  instanceIndex,
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

import { isPackedBuild, isPackedTreeBuild, toPackedNormal, toPackedTreeRigidity } from "#/shader/packed-vertex.tsl";
import { EVertexAttribute, INSTANCE_MATRIX_COLUMNS } from "#/shader/vertex-attribute";
import { STATIC_PLACE_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { TreeWindUniforms } from "#/uniforms/tree-wind-uniforms";

// Where a vertex stands: in each place its instanced attributes name, where the static draw buffers put it - by its
// slot's matrix, or by the place the cull listed for its instance - or where its object's matrix puts it. The buffer
// placed ones read no uniform of the object's own, so three refreshes nothing per object for them; which one a shader
// is built for follows from its geometry's attributes, as three's shader cache does.

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

/**
 * @param builder - The builder of the shader in question.
 * @returns Whether the geometry it builds for is an instanced static draw, each instance a place the cull listed.
 */
export function isListedBuild(builder: NodeBuilder): boolean {
  return Boolean(builder.geometry?.hasAttribute(EVertexAttribute.INSTANCE_LIST));
}

/**
 * @param builder - The builder of the shader in question.
 * @returns Whether the buffers every static draw shares place what it builds for, so a replay refreshes nothing.
 */
export function isBufferPlacedBuild(builder: NodeBuilder): boolean {
  return isStaticBuild(builder) || isListedBuild(builder);
}

/** A place's transform, from the four columns its instanced attributes carry. */
function toInstanceMatrix(): Node<"mat4"> {
  const [x, y, z, w] = INSTANCE_MATRIX_COLUMNS.map((column: string) => attribute<"vec4">(column, "vec4"));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/** A static draw's matrix, from its slot in the shared buffers, read once into a variable. */
function toStaticMatrix(buffers: StaticDrawBuffers): Node<"mat4"> {
  const first: Node<"uint"> = attribute<"uint">(EVertexAttribute.STATIC_SLOT, "uint").mul(4);
  const [x, y, z, w] = [0, 1, 2, 3].map((column: number) => buffers.modelColumns.element(first.add(column)));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/** The place the cull listed for the instance being drawn. */
function toListedPlace(buffers: StaticDrawBuffers): Node<"uint"> {
  return buffers.visiblePlaces.element(instanceIndex) as unknown as Node<"uint">;
}

/**
 * @param buffers - What static draws are placed by.
 * @returns The impostor the place listed for the instance being drawn is, which an impostor surface draws.
 */
export function toListedImpostor(buffers: StaticDrawBuffers): Node<"uint"> {
  return (
    buffers.placeColumns.element(toListedPlace(buffers).mul(STATIC_PLACE_COLUMNS).add(4)) as unknown as Node<"vec4">
  ).z.toUint();
}

/** An instanced static draw's matrix, from the place listed for its instance. */
function toListedMatrix(buffers: StaticDrawBuffers): Node<"mat4"> {
  const first: Node<"uint"> = toListedPlace(buffers).mul(STATIC_PLACE_COLUMNS);
  const [x, y, z, w] = [0, 1, 2, 3].map((column: number) => buffers.placeColumns.element(first.add(column)));

  return mat4(x, y, z, w) as unknown as Node<"mat4">;
}

/**
 * @param buffers - What static draws are placed by.
 * @returns The hemisphere scale and offset of the place listed for the instance being drawn.
 */
export function toListedHemiTerms(buffers: StaticDrawBuffers): Node<"vec2"> {
  return buffers.placeColumns.element(toListedPlace(buffers).mul(STATIC_PLACE_COLUMNS).add(4)).xy as Node<"vec2">;
}

/** What places a buffer placed build: its slot's matrix, or the listed place's. */
function toBufferMatrix(builder: NodeBuilder, buffers: StaticDrawBuffers): Node<"mat4"> {
  return isListedBuild(builder) ? toListedMatrix(buffers) : toStaticMatrix(buffers);
}

/** The vertex's normal in its geometry's own space: the engine's packed one, or three's float one. */
function toLocalNormal(builder: NodeBuilder): Node<"vec3"> {
  return isPackedBuild(builder) ? toPackedNormal() : normalLocal;
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
  return cameraViewMatrix.mul(vec4(toBufferPlacedWorld(builder, buffers, wind), 1)).xyz;
}

/**
 * @param builder - The builder of a buffer placed shader.
 * @param buffers - What static draws are placed by.
 * @param wind - How the trees sway.
 * @param isPrevious - Whether it is where the vertex stood the frame before: a static draw stands still, and only a
 *   tree's sway moves it.
 * @returns A static draw's vertex in the world.
 */
export function toBufferPlacedWorld(
  builder: NodeBuilder,
  buffers: StaticDrawBuffers,
  wind: TreeWindUniforms,
  isPrevious: boolean = false
): Node<"vec3"> {
  const matrix: Node<"mat4"> = toBufferMatrix(builder, buffers);
  const world: Node<"vec3"> = matrix.mul(vec4(positionLocal, 1)).xyz;

  if (!isPackedTreeBuild(builder)) {
    return world;
  }

  return isPrevious
    ? toSwayed(world, matrix, wind.previousWind, wind.previousWave)
    : toSwayed(world, matrix, wind.wind, wind.wave);
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

/** `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine. */
function toCyclic(phase: Node<"float">): Node<"float"> {
  const f: Node<"float"> = fract(phase).mul(2.8284271).sub(1.4142136);

  return f.mul(f).sub(1);
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
