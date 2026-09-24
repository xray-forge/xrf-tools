import { Nullable } from "@xrf/types";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import {
  SectorDescription,
  SectorGeometry,
  SectorSkip,
  SectorSurface,
  VisualBounds,
  VisualSection,
} from "@/core/ipc/types/xrf-visual";
import { getLevelSurfaceRender, ILevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";

/**
 * One packed mesh as views over the buffer it arrived in: positions as floats, the rest the vertex xrLC wrote.
 */
export interface ISectorGeometryViews {
  vertexCount: number;
  indexCount: number;
  positions: Float32Array;
  /** Four bytes a vertex: the packed normal, the hemisphere term fourth. */
  normals: Nullable<Uint8Array>;
  /** Four bytes a vertex: the packed tangent, the low byte of the base `u` fourth. */
  tangents: Nullable<Uint8Array>;
  /** Four bytes a vertex: the packed binormal, the low byte of the base `v` fourth. */
  binormals: Nullable<Uint8Array>;
  /** The base coordinate as shorts, `uvComponents` a vertex. */
  uvs: Nullable<Int16Array>;
  /** Shorts the base coordinate takes a vertex: two, or a tree's four. */
  uvComponents: number;
  /** The second uv set a lightmapped surface samples its baked lighting with, as shorts. */
  lightmapUvs: Nullable<Int16Array>;
  /** Thirty-two bit, unlike a model's. */
  indices: Uint32Array;
}

/** One draw of a sector's own geometry: the range to draw, and the surface it is drawn with. */
export interface ISectorSectionViews {
  surface: SectorSurface;
  /** What that surface's blender compiles to, joined from the level's shader table as the sector arrives. */
  render: ILevelSurfaceRender;
  /** Drawables this section draws, by their index in the visuals run, for inspection rather than for drawing. */
  drawables: Array<number>;
  /** What its own vertices span, measured by the packer, or null where it reaches none. */
  bounds: Nullable<VisualBounds>;
  start: number;
  count: number;
  triangleCount: number;
}

/** One mesh a sector stands in many places, as views over the buffer it arrived in. */
export interface ISectorInstanceViews {
  surface: SectorSurface;
  /** What that surface's blender compiles to, joined the same way a section's is. */
  render: ILevelSurfaceRender;
  drawables: Array<number>;
  geometry: ISectorGeometryViews;
  instanceCount: number;
  /** Sixteen floats for each place the mesh stands, in the order a renderer uploads a matrix in. */
  transforms: Float32Array;
  /** Two floats for each place: what scales and then offsets its vertices' hemisphere term. */
  hemi: Float32Array;
}

/**
 * One packed sector as views over the single buffer it arrived in, plus the draws that consume them.
 */
export interface ISectorViews {
  sector: number;
  bufferLength: number;
  /** What the packed positions span, measured by the packer, or null where the sector packed nothing. */
  bounds: Nullable<VisualBounds>;
  geometry: ISectorGeometryViews;
  sections: Array<ISectorSectionViews>;
  instances: Array<ISectorInstanceViews>;
  /** Drawables the packer could not read, named so a viewer can say what is missing rather than quietly omit it. */
  skipped: Array<SectorSkip>;
}

/**
 * Views rather than copies: the whole point of transferring one buffer is that the attributes are used where they
 * landed. Byte offsets are aligned by the packer, which is what makes these constructors legal at all.
 */
function toFloatView(buffer: ArrayBuffer, section: VisualSection): Float32Array {
  return new Float32Array(buffer, section.byteOffset, section.byteLength / Float32Array.BYTES_PER_ELEMENT);
}

function toOptionalByteView(buffer: ArrayBuffer, section: Nullable<VisualSection>): Nullable<Uint8Array> {
  return section ? new Uint8Array(buffer, section.byteOffset, section.byteLength) : null;
}

function toOptionalShortView(buffer: ArrayBuffer, section: Nullable<VisualSection>): Nullable<Int16Array> {
  return section ? new Int16Array(buffer, section.byteOffset, section.byteLength / Int16Array.BYTES_PER_ELEMENT) : null;
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
    binormals: toOptionalByteView(buffer, geometry.binormals),
    indexCount: geometry.indexCount,
    indices: toIndexView(buffer, geometry.indices),
    lightmapUvs: toOptionalShortView(buffer, geometry.lightmapUvs),
    normals: toOptionalByteView(buffer, geometry.normals),
    positions: toFloatView(buffer, geometry.positions),
    tangents: toOptionalByteView(buffer, geometry.tangents),
    uvComponents: geometry.uvComponents,
    uvs: toOptionalShortView(buffer, geometry.uvs),
    vertexCount: geometry.vertexCount,
  };
}

/**
 * Turns one packed sector and the bytes it was described against into views a renderer can upload.
 *
 * @param description - What `open_sector` reported about the pack.
 * @param buffer - The bytes `read_sector` served for that same pack.
 * @param surfaces - How the renderer draws each row of the level's shader table, from the open.
 * @returns Views over the buffer, and the draws that consume them.
 */
export function createSectorViews(
  description: SectorDescription,
  buffer: ArrayBuffer,
  surfaces: ReadonlyArray<XraySurfaceDescriptor> = []
): ISectorViews {
  if (buffer.byteLength !== description.bufferLength) {
    throw new Error(
      `Sector buffer is ${buffer.byteLength} bytes but its description covers ${description.bufferLength}. ` +
        "The description and the buffer came from different packs."
    );
  }

  return {
    bounds: description.bounds,
    bufferLength: description.bufferLength,
    geometry: toGeometryViews(buffer, description.geometry),
    instances: description.instances.map((group) => ({
      drawables: group.drawables,
      geometry: toGeometryViews(buffer, group.geometry),
      hemi: toFloatView(buffer, group.hemi),
      instanceCount: group.instanceCount,
      render: getLevelSurfaceRender(surfaces, group.surface.shaderId),
      surface: group.surface,
      transforms: toFloatView(buffer, group.transforms),
    })),
    sections: description.sections.map((section) => ({
      bounds: section.bounds,
      count: section.draw.count,
      drawables: section.drawables,
      render: getLevelSurfaceRender(surfaces, section.surface.shaderId),
      start: section.draw.start,
      surface: section.surface,
      triangleCount: section.draw.count / 3,
    })),
    sector: description.sector,
    skipped: description.skipped,
  };
}
