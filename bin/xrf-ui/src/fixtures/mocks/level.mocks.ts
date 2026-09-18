import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { SectorDescription, SectorOutline, SectorSection } from "@/core/ipc/types/xrf-visual";
import { mockVisualBounds, MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

/**
 * A sector packed the way the rust packer packs one: positions and thirty-two bit indices in one buffer, and a
 * section naming the surface that draws them.
 *
 * @param buffer - Buffer the sections are written into, so the offsets a test reads are real ones.
 * @param overrides - Fields to replace on the description.
 * @returns A description covering exactly what was written.
 */
export function mockSectorDescription(
  buffer: MockVisualBuffer,
  overrides: Partial<SectorDescription> = {}
): SectorDescription {
  const positions = buffer.pushFloats([0, 0, 0, 1, 0, 0, 0, 1, 0]);
  const indices = buffer.pushIndices32([0, 1, 2]);

  return {
    binormals: null,
    bounds: mockVisualBounds(),
    bufferLength: buffer.byteLength,
    colors: null,
    hemi: null,
    indexCount: 3,
    indices,
    lightmapCoordinates: null,
    normals: null,
    positions,
    sections: [mockSectorSection()],
    sector: 0,
    skipped: [],
    tangents: null,
    textureCoordinates: null,
    vertexCount: 3,
    ...overrides,
  };
}

/**
 * One draw of a sector, dressed by a shader table entry.
 *
 * @param overrides - Fields to replace on the section.
 * @returns A section fixture drawing one triangle.
 */
export function mockSectorSection(overrides: Partial<SectorSection> = {}): SectorSection {
  return {
    draw: { start: 0, count: 3 },
    drawables: [1],
    shaderId: 1,
    shaderName: "default",
    textureName: "stone",
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
    hasSun: true,
    lights: 2,
    portals: 0,
    sectors: [mockSectorOutline()],
    shaderEntries: 4,
    source: { kind: "directory", path: "C:\\levels\\zaton" },
    visuals: 2,
    xrlcQuality: 1,
    xrlcVersion: 14,
    ...overrides,
  };
}
