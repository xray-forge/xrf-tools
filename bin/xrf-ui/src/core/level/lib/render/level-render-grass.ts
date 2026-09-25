import { ERendererDraw, IRendererGrass, IRendererGrassModel } from "@xrf/renderer";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { DetailsModel, VisualSection } from "@/core/ipc/types/xrf-visual";
import { ILevelGrassDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ILevelSurfaceRender, toLevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";

/**
 * A level's grass as the renderer plants it, over a copy of the pack: the renderer takes what it is handed, and the
 * loader keeps the pack for a renderer started later.
 *
 * @param grass - The grass the loader holds.
 * @returns What the renderer plants.
 */
export function toLevelRendererGrass(grass: ILevelGrassDelivery): IRendererGrass {
  const buffer: ArrayBuffer = grass.buffer.slice(0);
  const { details, surfaces } = grass.description;

  function words(section: VisualSection): Uint32Array {
    return new Uint32Array(buffer, section.byteOffset, section.byteLength / 4);
  }

  function floats(section: VisualSection): Float32Array {
    return new Float32Array(buffer, section.byteOffset, section.byteLength / 4);
  }

  return {
    bins: words(details.bins),
    grid: words(details.grid),
    models: details.models.map((model: DetailsModel, index: number): IRendererGrassModel => ({
      height: model.height ?? 0,
      indices: new Uint16Array(buffer, model.indices.byteOffset, model.indices.byteLength / 2),
      isWaving: model.isWaving,
      maxScale: model.maxScale ?? 1,
      minScale: model.minScale ?? 1,
      positions: floats(model.positions),
      radius: model.radius ?? 0,
      surface: toGrassSurface(model, surfaces[index] ?? null),
      uvs: floats(model.uvs),
    })),
    offsetX: details.offsetX,
    offsetZ: details.offsetZ,
    sizeX: details.sizeX,
    sizeZ: details.sizeZ,
    slots: words(details.slots),
    triangles: floats(details.triangles),
  };
}

/** A model dressed as `CDetail::Load` binds it: its shader over its texture, cut out as the blender says. */
function toGrassSurface(model: DetailsModel, descriptor: XraySurfaceDescriptor | null): IRendererGrassModel["surface"] {
  const render: ILevelSurfaceRender = toLevelSurfaceRender(descriptor);

  return {
    alphaReference: render.alphaReference ?? undefined,
    draw: ERendererDraw.CUT_OUT,
    textures: { base: model.texture || undefined },
  };
}
