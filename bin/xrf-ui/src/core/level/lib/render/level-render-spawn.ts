import { IRendererGeometry, IRendererObject, IRendererSurface } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { LevelSpawnModelDescription, LevelSpawnModelsDescription, LevelSpawnPlacement } from "@/core/ipc/types/xrf-app";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualTransform } from "@/core/ipc/types/xrf-visual";
import { toLevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { createVisualViews, IVisualModelViews, IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";

/** Floats one bone's transform takes: its basis, then its translation. */
const FLOATS_PER_BONE: number = 12;

/** A level's spawned models handed to whatever draws them: what the backend said, and each model's pack by its name. */
export interface ILevelSpawnModelsDelivery {
  description: LevelSpawnModelsDescription;
  buffers: ReadonlyMap<string, ArrayBuffer>;
}

/** What one submesh of one model puts into the renderer, under one key. */
export interface ILevelSpawnPart {
  key: string;
  geometry: IRendererGeometry;
  surface: IRendererSurface;
  object: IRendererObject;
}

/**
 * The parts a level's spawned models draw as: each submesh of each model posed as the model stands still, once, and
 * drawn in every place an object of it stands.
 *
 * @param delivery - The models and where they stand.
 * @param isTextured - Whether surfaces draw their textures.
 * @returns Every part, by its key.
 */
export function toLevelSpawnParts(delivery: ILevelSpawnModelsDelivery, isTextured: boolean): Array<ILevelSpawnPart> {
  const { models, placements } = delivery.description;

  return models.flatMap((model: LevelSpawnModelDescription, index: number): Array<ILevelSpawnPart> => {
    const buffer: ArrayBuffer | undefined = delivery.buffers.get(model.name);
    const standing: Array<LevelSpawnPlacement> = placements.filter((it: LevelSpawnPlacement) => it.model === index);

    if (!buffer || !standing.length) {
      return [];
    }

    const views: IVisualModelViews = createVisualViews(model.description, buffer);
    const transforms: Float32Array = toInstanceTransforms(standing);

    return views.submeshes.map((submesh: IVisualSubmeshViews) => {
      const key: string = toLevelSpawnKey(index, submesh.index);
      const texture: Nullable<string> = model.description.submeshes[submesh.index]?.textureName ?? null;

      return {
        geometry: toPosedGeometry(submesh, views.skeletonBinds, model.rest?.map((it) => it ?? 0) ?? null),
        key,
        object: { geometry: key, instances: { transforms: transforms.slice() }, surfaces: [key] },
        surface: toSpawnSurface(model.surfaces[submesh.index] ?? null, isTextured ? texture : null),
      };
    });
  });
}

/**
 * @param model - A model, by its index among the description's.
 * @param submesh - One of its submeshes.
 * @returns The key its geometry, surface and object share.
 */
export function toLevelSpawnKey(model: number, submesh: number): string {
  return `spawn:${model}:${submesh}`;
}

/**
 * A submesh as its model stands still: every vertex moved by the bones it hangs from, from their bind to their rest,
 * weighted as the skin weights it; a submesh without skin or a model without a rest pose stands as authored.
 *
 * @param submesh - The submesh's views.
 * @param binds - Every bone's bind transform, twelve floats each, or null.
 * @param rest - Every bone's rest transform, twelve floats each, or null.
 * @returns Its geometry, drawn plainly: the pose is baked in.
 */
export function toPosedGeometry(
  submesh: IVisualSubmeshViews,
  binds: Nullable<Float32Array>,
  rest: Nullable<ReadonlyArray<number>>
): IRendererGeometry {
  const positions: Float32Array = submesh.positions.slice();
  const normals: Float32Array = submesh.normals.slice();
  const tangents: Float32Array = submesh.tangents.slice();
  const binormals: Float32Array = submesh.binormals.slice();

  if (submesh.skinIndices && submesh.skinWeights && binds && rest) {
    const skins: Array<Float32Array> = toSkinMatrices(binds, rest);

    for (let vertex: number = 0; vertex < positions.length / 3; vertex += 1) {
      skinVertex(vertex, submesh, skins, positions, [normals, tangents, binormals]);
    }
  }

  return {
    binormal: binormals,
    groups: [{ count: submesh.indices.length, slot: 0, start: 0 }],
    index: submesh.indices.slice(),
    normal: normals,
    position: positions,
    tangent: tangents,
    uv: submesh.uvs.slice(),
  };
}

/** Sixteen floats a placement, column major: the object's basis and place, as the renderer stands instances. */
function toInstanceTransforms(placements: ReadonlyArray<LevelSpawnPlacement>): Float32Array {
  const transforms: Float32Array = new Float32Array(placements.length * 16);

  placements.forEach(({ transform }: { transform: VisualTransform }, index: number) => {
    const { i, j, k, c } = transform;

    transforms.set(
      [
        i.x ?? 0,
        i.y ?? 0,
        i.z ?? 0,
        0,
        j.x ?? 0,
        j.y ?? 0,
        j.z ?? 0,
        0,
        k.x ?? 0,
        k.y ?? 0,
        k.z ?? 0,
        0,
        c.x ?? 0,
        c.y ?? 0,
        c.z ?? 0,
        1,
      ],
      index * 16
    );
  });

  return transforms;
}

/** Each bone's rest transform after the inverse of its bind: what takes a bound vertex to where the bone rests. */
function toSkinMatrices(binds: Float32Array, rest: ReadonlyArray<number>): Array<Float32Array> {
  const count: number = Math.min(binds.length, rest.length) / FLOATS_PER_BONE;

  return Array.from({ length: count }, (_, bone: number) =>
    compose(
      rest.slice(bone * FLOATS_PER_BONE, (bone + 1) * FLOATS_PER_BONE),
      invert(binds.subarray(bone * FLOATS_PER_BONE, (bone + 1) * FLOATS_PER_BONE))
    )
  );
}

/** A rigid transform's inverse: its basis transposed, its translation turned back through it. */
function invert(transform: ArrayLike<number>): Float32Array {
  const [ix, iy, iz, jx, jy, jz, kx, ky, kz, cx, cy, cz] = Array.from(transform);

  return new Float32Array([
    ix,
    jx,
    kx,
    iy,
    jy,
    ky,
    iz,
    jz,
    kz,
    -(ix * cx + iy * cy + iz * cz),
    -(jx * cx + jy * cy + jz * cz),
    -(kx * cx + ky * cy + kz * cz),
  ]);
}

/** `outer` after `inner`: a point through `inner`, then through `outer`. */
function compose(outer: ArrayLike<number>, inner: ArrayLike<number>): Float32Array {
  const result: Float32Array = new Float32Array(FLOATS_PER_BONE);

  for (let axis: number = 0; axis < 3; axis += 1) {
    rotate(outer, inner[axis * 3], inner[axis * 3 + 1], inner[axis * 3 + 2], result, axis * 3);
  }

  rotate(outer, inner[9], inner[10], inner[11], result, 9);
  result[9] += outer[9];
  result[10] += outer[10];
  result[11] += outer[11];

  return result;
}

/** A direction through a transform's basis, written at an offset. */
function rotate(transform: ArrayLike<number>, x: number, y: number, z: number, out: Float32Array, at: number): void {
  out[at] = transform[0] * x + transform[3] * y + transform[6] * z;
  out[at + 1] = transform[1] * x + transform[4] * y + transform[7] * z;
  out[at + 2] = transform[2] * x + transform[5] * y + transform[8] * z;
}

/** One vertex moved by its weighted bones, and its directions turned with it. */
function skinVertex(
  vertex: number,
  submesh: IVisualSubmeshViews,
  skins: ReadonlyArray<Float32Array>,
  positions: Float32Array,
  directions: ReadonlyArray<Float32Array>
): void {
  const at: number = vertex * 3;
  const moved: Float32Array = new Float32Array(3 + directions.length * 3);
  const scratch: Float32Array = new Float32Array(3);

  for (let link: number = 0; link < 4; link += 1) {
    const weight: number = (submesh.skinWeights as Float32Array)[vertex * 4 + link];
    const skin: Float32Array | undefined = skins[(submesh.skinIndices as Uint16Array)[vertex * 4 + link]];

    if (!weight || !skin) {
      continue;
    }

    rotate(skin, submesh.positions[at], submesh.positions[at + 1], submesh.positions[at + 2], scratch, 0);
    moved[0] += (scratch[0] + skin[9]) * weight;
    moved[1] += (scratch[1] + skin[10]) * weight;
    moved[2] += (scratch[2] + skin[11]) * weight;

    directions.forEach((direction: Float32Array, index: number) => {
      rotate(skin, direction[at], direction[at + 1], direction[at + 2], scratch, 0);
      moved[3 + index * 3] += scratch[0] * weight;
      moved[3 + index * 3 + 1] += scratch[1] * weight;
      moved[3 + index * 3 + 2] += scratch[2] * weight;
    });
  }

  positions.set(moved.subarray(0, 3), at);
  directions.forEach((direction: Float32Array, index: number) => {
    const x: number = moved[3 + index * 3];
    const y: number = moved[3 + index * 3 + 1];
    const z: number = moved[3 + index * 3 + 2];
    const length: number = Math.hypot(x, y, z) || 1;

    direction.set([x / length, y / length, z / length], at);
  });
}

/** A submesh dressed as its shader resolved: its base texture, cut out or blended as the blender says. */
function toSpawnSurface(descriptor: Nullable<XraySurfaceDescriptor>, texture: Nullable<string>): IRendererSurface {
  const render = toLevelSurfaceRender(descriptor);

  return {
    alphaReference: render.alphaReference,
    draw: render.draw,
    isLit: render.isLit,
    textures: { base: texture || undefined },
  };
}
