import { SectorDescription, SectorSection, VisualSection } from "@/core/ipc/types/xrf-visual";
import { Nullable } from "@/lib/types/general";

/**
 * One draw of a sector: the indices to draw, and the surface they are drawn with.
 */
export interface ISectorSectionViews {
  shaderId: number;
  shaderName: Nullable<string>;
  textureName: Nullable<string>;
  /** Drawables this section draws, by their index in the visuals run, for inspection rather than for drawing. */
  drawables: Array<number>;
  start: number;
  count: number;
  triangleCount: number;
}

/**
 * One packed sector as views over the single buffer it arrived in, plus the draws that consume them.
 */
export interface ISectorViews {
  sector: number;
  vertexCount: number;
  indexCount: number;
  positions: Float32Array;
  /** Absent when no declaration in the sector carried one, as a positions-only fast path does not. */
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
  sections: Array<ISectorSectionViews>;
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
    binormals: toOptionalFloatView(buffer, description.binormals),
    colors: toOptionalFloatView(buffer, description.colors),
    hemi: toOptionalFloatView(buffer, description.hemi),
    indexCount: description.indexCount,
    indices: toIndexView(buffer, description.indices),
    lightmapUvs: toOptionalFloatView(buffer, description.lightmapCoordinates),
    normals: toOptionalFloatView(buffer, description.normals),
    positions: toFloatView(buffer, description.positions),
    sector: description.sector,
    sections: description.sections.map((section: SectorSection) => ({
      count: section.draw.count,
      drawables: section.drawables,
      shaderId: section.shaderId,
      shaderName: section.shaderName,
      start: section.draw.start,
      textureName: section.textureName,
      triangleCount: section.draw.count / 3,
    })),
    skipped: description.skipped.map((skip) => ({ drawable: skip.drawable, reason: skip.reason })),
    tangents: toOptionalFloatView(buffer, description.tangents),
    uvs: toOptionalFloatView(buffer, description.textureCoordinates),
    vertexCount: description.vertexCount,
  };
}

/** Triangles a sector draws in total, which is what a viewer reports rather than its index count. */
export function countSectorTriangles(views: ISectorViews): number {
  return views.sections.reduce((total: number, section: ISectorSectionViews) => total + section.triangleCount, 0);
}
