import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { LevelEntry, LevelTextureReference, SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import {
  SectorDescription,
  SectorGeometry,
  SectorInstanceGroup,
  SectorOutline,
  SectorSection,
  SectorSurface,
} from "@/core/ipc/types/xrf-visual";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { mockVisualBounds, MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

/**
 * One packed mesh: positions and thirty-two bit indices written into the buffer.
 *
 * @param buffer - Buffer the attributes are written into, so the offsets a test reads are real ones.
 * @returns Where they landed.
 */
export function mockSectorGeometry(buffer: MockVisualBuffer): SectorGeometry {
  const positions = buffer.pushFloats([0, 0, 0, 1, 0, 0, 0, 1, 0]);
  const indices = buffer.pushIndices32([0, 1, 2]);

  return {
    binormals: null,
    indexCount: 3,
    indices,
    lightmapUvs: null,
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
    shaderEntries: 4,
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
