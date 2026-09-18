import { SectorDescription, SectorGeometry, SectorSurface, VisualSection } from "@/core/ipc/types/xrf-visual";
import { Nullable } from "@/lib/types/general";

/**
 * One packed mesh as views over the buffer it arrived in.
 */
export interface ISectorGeometryViews {
  vertexCount: number;
  indexCount: number;
  positions: Float32Array;
  normals: Nullable<Float32Array>;
  tangents: Nullable<Float32Array>;
  binormals: Nullable<Float32Array>;
  uvs: Nullable<Float32Array>;
  /** The second uv set a lightmapped surface samples its baked lighting with. */
  lightmapUvs: Nullable<Float32Array>;
  /** Baked vertex colour, for the surfaces xrLC lit that way instead of with a lightmap. */
  colors: Nullable<Float32Array>;
  /** Hemisphere occlusion, one float per vertex, which rides in the normal and is present with it. */
  hemi: Nullable<Float32Array>;
  /** Thirty-two bit, unlike a model's. */
  indices: Uint32Array;
}

/** One draw of a sector's own geometry: the range to draw, and the surface it is drawn with. */
export interface ISectorSectionViews {
  surface: SectorSurface;
  /** Drawables this section draws, by their index in the visuals run, for inspection rather than for drawing. */
  drawables: Array<number>;
  start: number;
  count: number;
  triangleCount: number;
}

/** One mesh a sector stands in many places, as views over the buffer it arrived in. */
export interface ISectorInstanceViews {
  surface: SectorSurface;
  drawables: Array<number>;
  geometry: ISectorGeometryViews;
  instanceCount: number;
  /** Sixteen floats for each place the mesh stands, in the order a renderer uploads a matrix in. */
  transforms: Float32Array;
}

/**
 * One packed sector as views over the single buffer it arrived in, plus the draws that consume them.
 */
export interface ISectorViews {
  sector: number;
  bufferLength: number;
  geometry: ISectorGeometryViews;
  sections: Array<ISectorSectionViews>;
  instances: Array<ISectorInstanceViews>;
  /** Drawables the packer could not read, named so a viewer can say what is missing rather than quietly omit it. */
  skipped: Array<{ drawable: number; reason: string }>;
}

/**
 * Views rather than copies: the whole point of transferring one buffer is that the attributes are used where they
 * landed. Byte offsets are aligned by the packer, which is what makes these constructors legal at all.
 */
function toFloatView(buffer: ArrayBuffer, section: VisualSection): Float32Array {
  return new Float32Array(buffer, section.byteOffset, section.byteLength / Float32Array.BYTES_PER_ELEMENT);
}

function toOptionalFloatView(buffer: ArrayBuffer, section: Nullable<VisualSection>): Nullable<Float32Array> {
  return section ? toFloatView(buffer, section) : null;
}

function toIndexView(buffer: ArrayBuffer, section: VisualSection): Uint32Array {
  return new Uint32Array(buffer, section.byteOffset, section.byteLength / Uint32Array.BYTES_PER_ELEMENT);
}

/**
 * Turns one packed mesh's described sections into views over the bytes they describe.
 *
 * @param buffer - The sector's buffer.
 * @param geometry - Where the mesh's attributes sit in it.
 * @returns Views over each attribute the mesh carries.
 */
function toGeometryViews(buffer: ArrayBuffer, geometry: SectorGeometry): ISectorGeometryViews {
  return {
    binormals: toOptionalFloatView(buffer, geometry.binormals),
    colors: toOptionalFloatView(buffer, geometry.colors),
    hemi: toOptionalFloatView(buffer, geometry.hemi),
    indexCount: geometry.indexCount,
    indices: toIndexView(buffer, geometry.indices),
    lightmapUvs: toOptionalFloatView(buffer, geometry.lightmapCoordinates),
    normals: toOptionalFloatView(buffer, geometry.normals),
    positions: toFloatView(buffer, geometry.positions),
    tangents: toOptionalFloatView(buffer, geometry.tangents),
    uvs: toOptionalFloatView(buffer, geometry.textureCoordinates),
    vertexCount: geometry.vertexCount,
  };
}

/**
 * Turns one packed sector and the bytes it was described against into views a renderer can upload.
 *
 * @param description - What `open_sector` reported about the pack.
 * @param buffer - The bytes `read_sector` served for that same pack.
 * @returns Views over the buffer, and the draws that consume them.
 */
export function createSectorViews(description: SectorDescription, buffer: ArrayBuffer): ISectorViews {
  if (buffer.byteLength !== description.bufferLength) {
    throw new Error(
      `Sector buffer is ${buffer.byteLength} bytes but its description covers ${description.bufferLength}. ` +
        "The description and the buffer came from different packs."
    );
  }

  return {
    bufferLength: description.bufferLength,
    geometry: toGeometryViews(buffer, description.geometry),
    instances: description.instances.map((group) => ({
      drawables: group.drawables,
      geometry: toGeometryViews(buffer, group.geometry),
      instanceCount: group.instanceCount,
      surface: group.surface,
      transforms: toFloatView(buffer, group.transforms),
    })),
    sections: description.sections.map((section) => ({
      count: section.draw.count,
      drawables: section.drawables,
      start: section.draw.start,
      surface: section.surface,
      triangleCount: section.draw.count / 3,
    })),
    sector: description.sector,
    skipped: description.skipped.map((skip) => ({ drawable: skip.drawable, reason: skip.reason })),
  };
}

/**
 * Every texture reference a sector names, base textures and lightmaps alike, from both kinds of surface.
 *
 * @param views - The sector.
 * @returns Its references, without repeats.
 */
export function listSectorTextures(views: ISectorViews): Array<string> {
  const references: Set<string> = new Set();

  for (const { surface } of [...views.sections, ...views.instances]) {
    if (surface.textureName) {
      references.add(surface.textureName);
    }

    for (const lightmap of surface.lightmaps) {
      references.add(lightmap);
    }
  }

  return Array.from(references);
}

/** Triangles a sector draws in total, instanced meshes counted once for every place they stand. */
export function countSectorTriangles(views: ISectorViews): number {
  const baked: number = views.sections.reduce(
    (total: number, section: ISectorSectionViews) => total + section.triangleCount,
    0
  );

  return views.instances.reduce(
    (total: number, group: ISectorInstanceViews) => total + (group.geometry.indexCount / 3) * group.instanceCount,
    baked
  );
}
