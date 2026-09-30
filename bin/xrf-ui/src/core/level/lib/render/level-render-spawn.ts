import {
  IRendererClusters,
  IRendererGeometry,
  IRendererGeometryGroup,
  IRendererObject,
  IRendererSurface,
  RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE,
  TRendererColor,
} from "@xrf/renderer";
import { Maybe, Nullable, Optional } from "@xrf/types";

import { LevelSpawnObject } from "@/core/ipc/types/xrf-app";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualTransform } from "@/core/ipc/types/xrf-visual";
import { ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import { toPosedGeometry } from "@/core/level/lib/render/level-render-spawn-pose";
import { toLevelSurfaceColor } from "@/core/level/lib/render/level-render-surface";
import { ILevelSurfaceRender, toLevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { createVisualViews, IVisualModelViews, IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";

/** Unsigned integers one cluster's range takes: its first index, then three more. */
const WORDS_PER_CLUSTER: number = 4;

/** Vertices a sixteen-bit index can name. */
const SHORT_INDEX_VERTICES: number = 65536;

/** What a submesh is dressed as: its shader as it resolved, its base texture, and its colour without one. */
export interface ILevelSpawnDressing {
  descriptor: Nullable<XraySurfaceDescriptor>;
  texture: Nullable<string>;
  /** The model's own, so one model reads as one wherever it stands while textures are off. */
  color: TRendererColor;
}

/** One visual as the renderer draws it: its submeshes posed as it stands still in one geometry, a group each. */
export interface ILevelSpawnModelParts {
  geometry: IRendererGeometry;
  /** Each submesh's dressing, by the group slot it draws. */
  dressings: ReadonlyArray<ILevelSpawnDressing>;
}

/**
 * One visual of a level's spawned objects as the renderer draws it: every submesh posed as the model stands still,
 * once, and joined into one geometry, so the model is bounded, and dropped when small on screen, as the engine does a
 * visual.
 *
 * @param visual - The visual, by its index among the objects' visuals, which colours it.
 * @param model - Its model and pack.
 * @returns Its geometry and each group's dressing.
 */
export function toLevelSpawnModelParts(visual: number, model: ILevelSpawnModel): ILevelSpawnModelParts {
  const { description, buffer } = model;
  const views: IVisualModelViews = createVisualViews(description.description, buffer);
  const rest: Nullable<Array<number>> = description.rest?.map((it) => it ?? 0) ?? null;
  const submeshes: Array<IRendererGeometry> = views.submeshes.map((submesh: IVisualSubmeshViews) =>
    toPosedGeometry(submesh, views.skeletonBinds, rest)
  );

  return {
    dressings: views.submeshes.map((submesh: IVisualSubmeshViews) => ({
      color: toLevelSurfaceColor(visual),
      descriptor: description.surfaces[submesh.index] ?? null,
      texture: description.description.submeshes[submesh.index]?.textureName ?? null,
    })),
    geometry: joinGeometries(submeshes),
  };
}

/**
 * The objects standing as one visual, drawn as its geometry in every place one stands, each lit by its hemisphere
 * cube where the backend estimated one.
 *
 * @param geometry - The key its visual's geometry is put under.
 * @param surfaces - The keys its groups' surfaces are put under, by slot.
 * @param standing - The objects.
 * @param hemi - Each object's hemisphere cube, by its index among the level's spawned objects.
 * @returns What the renderer stands them as.
 */
export function toLevelSpawnObject(
  geometry: string,
  surfaces: ReadonlyArray<string>,
  standing: ReadonlyArray<LevelSpawnObject>,
  hemi: ReadonlyMap<number, ReadonlyArray<number>>
): IRendererObject {
  const cubes: Array<Maybe<ReadonlyArray<number>>> = standing.map((object: LevelSpawnObject) => hemi.get(object.index));

  return {
    geometry,
    instances: {
      // All or none: an object without a cube of its own would read the next one's.
      hemiCube: cubes.every(Boolean) ? toHemiCubes(cubes as Array<ReadonlyArray<number>>) : undefined,
      transforms: toInstanceTransforms(standing),
    },
    surfaces,
  };
}

/** Six floats an object, its cube's faces in their order. */
function toHemiCubes(cubes: ReadonlyArray<ReadonlyArray<number>>): Float32Array {
  const faces: Float32Array = new Float32Array(cubes.length * RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE);

  cubes.forEach((cube: ReadonlyArray<number>, index: number) =>
    faces.set(cube.slice(0, RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE), index * RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE)
  );

  return faces;
}

/** Sixteen floats an object, column major: its basis and place, as the renderer stands instances. */
function toInstanceTransforms(objects: ReadonlyArray<LevelSpawnObject>): Float32Array {
  const transforms: Float32Array = new Float32Array(objects.length * 16);

  objects.forEach(({ transform }: { transform: VisualTransform }, index: number) => {
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

/**
 * The submeshes of one model as one geometry, a group a submesh by its slot, each one's indices moved past the
 * vertices before it. Their clusters join too where every submesh has them; otherwise the renderer cuts its own.
 */
function joinGeometries(parts: ReadonlyArray<IRendererGeometry>): IRendererGeometry {
  const vertices: number = parts.reduce((sum: number, part: IRendererGeometry) => sum + part.position.length / 3, 0);
  const indices: number = parts.reduce((sum: number, part: IRendererGeometry) => sum + (part.index?.length ?? 0), 0);
  const index: Uint16Array | Uint32Array =
    vertices > SHORT_INDEX_VERTICES ? new Uint32Array(indices) : new Uint16Array(indices);
  const groups: Array<IRendererGeometryGroup> = [];
  let vertexStart: number = 0;
  let indexStart: number = 0;

  parts.forEach((part: IRendererGeometry, slot: number) => {
    const count: number = part.index?.length ?? 0;

    for (let at: number = 0; at < count; at += 1) {
      index[indexStart + at] = (part.index as Uint16Array | Uint32Array)[at] + vertexStart;
    }

    groups.push({ count, slot, start: indexStart });
    vertexStart += part.position.length / 3;
    indexStart += count;
  });

  return {
    binormal: joinAttribute(parts, "binormal"),
    clusters: joinClusters(parts, groups),
    groups,
    index,
    normal: joinAttribute(parts, "normal"),
    position: joinAttribute(parts, "position") as Float32Array,
    tangent: joinAttribute(parts, "tangent"),
    uv: joinAttribute(parts, "uv"),
  };
}

/** One float attribute of every part end to end, or none where a part lacks it. */
function joinAttribute(
  parts: ReadonlyArray<IRendererGeometry>,
  name: "position" | "normal" | "tangent" | "binormal" | "uv"
): Optional<Float32Array> {
  if (parts.some((part: IRendererGeometry) => !part[name])) {
    return undefined;
  }

  const joined: Float32Array = new Float32Array(
    parts.reduce((sum: number, part: IRendererGeometry) => sum + (part[name] as Float32Array).length, 0)
  );
  let at: number = 0;

  for (const part of parts) {
    joined.set(part[name] as Float32Array, at);
    at += (part[name] as Float32Array).length;
  }

  return joined;
}

/** Every part's clusters, each range's first index moved to where its group now starts. */
function joinClusters(
  parts: ReadonlyArray<IRendererGeometry>,
  groups: ReadonlyArray<IRendererGeometryGroup>
): Optional<IRendererClusters> {
  if (!parts.length || parts.some((part: IRendererGeometry) => !part.clusters)) {
    return undefined;
  }

  const clusters: Array<IRendererClusters> = parts.map((part: IRendererGeometry) => part.clusters as IRendererClusters);
  const ranges: Uint32Array = new Uint32Array(clusters.reduce((sum: number, it) => sum + it.ranges.length, 0));
  const spheres: Float32Array = new Float32Array(clusters.reduce((sum: number, it) => sum + it.spheres.length, 0));
  let at: number = 0;

  clusters.forEach((cluster: IRendererClusters, part: number) => {
    ranges.set(cluster.ranges, at);
    spheres.set(cluster.spheres, at);

    for (let word: number = at; word < at + cluster.ranges.length; word += WORDS_PER_CLUSTER) {
      ranges[word] += groups[part].start;
    }

    at += cluster.ranges.length;
  });

  return { ranges, spheres };
}

/**
 * A submesh dressed as its shader resolved, as a sector's surface is: its base texture with the bump pair and the
 * detail its descriptor binds beside it, cut out or blended as the blender says.
 *
 * @param dressing - Its shader and base texture.
 * @returns Its surface, which draws its colour while the settings draw no textures.
 */
export function toLevelSpawnSurface(dressing: ILevelSpawnDressing): IRendererSurface {
  const render: ILevelSurfaceRender = toLevelSurfaceRender(dressing.descriptor);

  return {
    alphaReference: render.alphaReference,
    color: dressing.color,
    detailScale: render.detail?.scale,
    draw: render.draw,
    isLit: render.isLit,
    isWallmark: render.isWallmark || undefined,
    material: render.material,
    textures: {
      base: dressing.texture || undefined,
      bump: render.bump?.bump,
      bumpCompanion: render.bump?.companion,
      detail: render.detail?.reference,
      detailBump: render.detail?.bump?.bump,
      detailBumpCompanion: render.detail?.bump?.companion,
    },
  };
}
