import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import {
  LevelDetailsDescription,
  LevelEntry,
  LevelTextureReference,
  SelectedLevelDescription,
} from "@/core/ipc/types/xrf-app";
import {
  DetailsDescription,
  DetailsModel,
  SectorDescription,
  SectorGeometry,
  SectorInstanceGroup,
  SectorOutline,
  SectorSection,
  SectorSurface,
  VisualSection,
} from "@/core/ipc/types/xrf-visual";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { mockSurfaceDescriptor, mockVisualBounds, MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

/**
 * One packed mesh: positions and thirty-two bit indices written into the buffer.
 *
 * @param buffer - Buffer the attributes are written into, so the offsets a test reads are real ones.
 * @returns Where they landed.
 */
export function mockSectorGeometry(buffer: MockVisualBuffer): SectorGeometry {
  const positions = buffer.pushFloats([0, 0, 0, 1, 0, 0, 0, 1, 0]);
  const indices = buffer.pushIndices32([0, 1, 2]);
  const ranges = buffer.pushIndices32([0, 1, 0, 0]);
  const spheres = buffer.pushFloats([0, 0, 0, 1]);

  return {
    binormals: null,
    clusters: { ranges, spheres },
    indexCount: 3,
    indices,
    lightmapUvs: null,
    colors: null,
    normals: null,
    positions,
    tangents: null,
    uvComponents: 0,
    uvs: null,
    vertexCount: 3,
  };
}

/**
 * How one part of a sector is dressed.
 *
 * @param overrides - Fields to replace on the surface.
 * @returns A surface fixture naming one shader table entry.
 */
export function mockSectorSurface(overrides: Partial<SectorSurface> = {}): SectorSurface {
  return {
    hemi: null,
    shaderId: 1,
    shaderName: "default",
    textureName: "stone",
    ...overrides,
  };
}

/**
 * One draw of a sector's own geometry.
 *
 * @param overrides - Fields to replace on the section.
 * @returns A section fixture drawing one triangle.
 */
export function mockSectorSection(overrides: Partial<SectorSection> = {}): SectorSection {
  return {
    bounds: null,
    draw: { start: 0, count: 3 },
    drawables: [1],
    surface: mockSectorSurface(),
    ...overrides,
  };
}

/**
 * A sector packed the way the rust packer packs one.
 *
 * @param buffer - Buffer the sections are written into, so the offsets a test reads are real ones.
 * @param overrides - Fields to replace on the description.
 * @returns A description covering exactly what was written.
 */
export function mockSectorDescription(
  buffer: MockVisualBuffer,
  overrides: Partial<SectorDescription> = {}
): SectorDescription {
  const geometry: SectorGeometry = mockSectorGeometry(buffer);

  return {
    bounds: mockVisualBounds(),
    geometry,
    impostors: null,
    instances: [],
    sections: [mockSectorSection()],
    sector: 0,
    skipped: [],
    ...overrides,
    // Last, so a caller that wrote more into the buffer still gets a length covering all of it.
    bufferLength: overrides.bufferLength ?? buffer.byteLength,
  };
}

/**
 * One mesh a sector stands in several places, packed once beside the transforms that place it.
 *
 * @param buffer - Buffer the mesh and its transforms are written into.
 * @param places - Where each copy stands along x.
 * @param overrides - Fields to replace on the group.
 * @returns A group covering exactly what was written.
 */
export function mockSectorInstanceGroup(
  buffer: MockVisualBuffer,
  places: Array<number>,
  overrides: Partial<SectorInstanceGroup> = {}
): SectorInstanceGroup {
  const geometry: SectorGeometry = mockSectorGeometry(buffer);
  const transforms = buffer.pushFloats(
    places.flatMap((at: number) => [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, at, 0, 0, 1])
  );
  const hemi = buffer.pushFloats(places.flatMap(() => [1, 0]));

  return {
    drawables: places.map((_, index: number) => index + 1),
    geometry,
    hemi,
    impostors: null,
    instanceCount: places.length,
    progressive: null,
    surface: mockSectorSurface(),
    transforms,
    ...overrides,
  };
}

/**
 * What one sector is before any of its geometry is read.
 *
 * @param overrides - Fields to replace on the outline.
 * @returns An outline fixture reaching one drawable.
 */
export function mockSectorOutline(overrides: Partial<SectorOutline> = {}): SectorOutline {
  return {
    bounds: mockVisualBounds(),
    drawables: 1,
    root: 0,
    sector: 0,
    ...overrides,
  };
}

/**
 * A level's grass packed the way the rust packer packs it: one model, and a two cell grid whose first cell plants over
 * one triangle.
 *
 * @param buffer - Buffer the sections are written into, so the offsets a test reads are real ones.
 * @param overrides - Fields to replace on the packed grass.
 * @returns A description covering exactly what was written.
 */
export function mockLevelDetailsDescription(
  buffer: MockVisualBuffer,
  overrides: Partial<DetailsDescription> = {}
): LevelDetailsDescription {
  const model: DetailsModel = {
    height: 2,
    indices: buffer.pushIndices([0, 2, 1]),
    isWaving: true,
    maxScale: 1.5,
    minScale: 0.5,
    positions: buffer.pushFloats([0, 0, 0, 1, 0, 0, 0, 2, 0]),
    radius: 1.2,
    texture: "detail\\grass",
    uvs: buffer.pushFloats([0, 1, 1, 1, 0.5, 0]),
  };
  const grid: VisualSection = buffer.pushIndices32([1, 0]);
  const slots: VisualSection = buffer.pushIndices32([1, 2, 3, 4, 0, 1]);
  const bins: VisualSection = buffer.pushIndices32([0]);
  const triangles: VisualSection = buffer.pushFloats([0, 1, 0, 2, 1, 0, 0, 1, 2]);

  return {
    details: {
      bins,
      grid,
      models: [model],
      offsetX: 3,
      offsetZ: -2,
      sizeX: 2,
      sizeZ: 1,
      slotCount: 1,
      slots,
      triangles,
      ...overrides,
      // Last, so a caller that wrote more into the buffer still gets a length covering all of it.
      bufferLength: overrides.bufferLength ?? buffer.byteLength,
    },
    surfaces: [mockSurfaceDescriptor({ shader: "details\\blend", textures: ["detail\\grass"] })],
    textures: [mockLevelTextureReference("detail\\grass")],
  };
}

/**
 * One texture a level's shader table names.
 *
 * @param name - The reference as the table spells it.
 * @param isPresent - Whether the roots hold anything for it.
 * @returns A reference fixture.
 */
export function mockLevelTextureReference(name: string, isPresent: boolean = true): LevelTextureReference {
  // `\\` rather than `\`, which in a template literal escapes the interpolation and gave every reference the one
  // logical path `textures${name}.dds`.
  return { logicalPath: isPresent ? `textures\\${name}.dds` : null, reference: name };
}

/**
 * One compiled level a picker lists.
 *
 * @param name - What the installation knows it by.
 * @param hasGeometry - Whether `level.geom` sits beside its bundle.
 * @returns An entry fixture.
 */
export function mockLevelEntry(name: string, hasGeometry: boolean = true): LevelEntry {
  return { hasGeometry, logicalPath: `levels\${name}`, name };
}

/**
 * A level as `open_level` describes one.
 *
 * @param overrides - Fields to replace on the description.
 * @returns A level description fixture naming one sector.
 */
export function mockSelectedLevelDescription(
  overrides: Partial<SelectedLevelDescription> = {}
): SelectedLevelDescription {
  return {
    bounds: mockVisualBounds(),
    drawables: 1,
    sun: null,
    hasSun: true,
    lights: 2,
    portals: 0,
    roots: createRoots([]),
    sectors: [mockSectorOutline()],
    start: null,
    shaderEntries: 4,
    sky: {
      environment: { logicalPath: "textures\\sky\\sky_7_cube#small.dds", reference: "sky\\sky_7_cube#small" },
      texture: { logicalPath: "textures\\sky\\sky_7_cube.dds", reference: "sky\\sky_7_cube" },
    },
    source: { kind: "directory", path: "C:\\levels\\zaton" },
    surfaces: [],
    textures: [],
    visuals: 2,
    xrlcQuality: 1,
    xrlcVersion: 14,
    ...overrides,
  };
}

/** What one reference came to, for a report fixture. */
export interface IMockLevelTexture {
  reason: Nullable<string>;
  upload: Nullable<string>;
}

/**
 * What a level's textures came to, as the renderer service would publish it.
 *
 * @param entries - What each reference came to.
 * @returns The report.
 */
export function mockLevelTextureReport(entries: Record<string, IMockLevelTexture> = {}): ILevelTextureReport {
  return {
    dressing: new Map(
      Object.entries(entries).map(([reference, loaded]) => [
        reference,
        {
          reason: loaded.reason,
          reference,
          state: loaded.reason ? ELevelSurfaceDressing.STOOD_IN : ELevelSurfaceDressing.UPLOADED,
          upload: loaded.upload,
        },
      ])
    ),
    problems: Object.entries(entries)
      .filter(([, loaded]) => loaded.reason)
      .map(([reference, loaded]) => ({ reason: loaded.reason as string, reference })),
    uploaded: Object.keys(entries).length,
  };
}

/**
 * Creates what a level view sets over the renderer's features: nothing, unless told.
 *
 * @param overrides - The groups the view sets.
 * @returns The view's feature options.
 */
export function mockLevelFeatureOptions(overrides: Partial<ILevelFeatureOptions> = {}): ILevelFeatureOptions {
  return { ambientOcclusion: {}, antialiasing: null, grass: {}, lights: {}, shadows: {}, water: {}, ...overrides };
}
