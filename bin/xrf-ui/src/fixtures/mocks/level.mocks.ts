import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import {
  ELevelSpawnCategory,
  LevelEntry,
  LevelSpawnObject,
  LevelTextureReference,
  SelectedLevelDescription,
} from "@/core/ipc/types/xrf-app";
import { EXrayEngine, EXrayEngineEvidence } from "@/core/ipc/types/xrf-engine-target";
import { EClsId } from "@/core/ipc/types/xrf-spawn";
import { SectorOutline } from "@/core/ipc/types/xrf-visual";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { mockVisualBounds, mockVisualTransform } from "@/fixtures/mocks/visual.mocks";

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
 * One spawned object the viewer draws: a crate at the origin, standing as the first visual.
 *
 * @param overrides - Fields to replace on the object.
 * @returns An object fixture.
 */
export function mockLevelSpawnObject(overrides: Partial<LevelSpawnObject> = {}): LevelSpawnObject {
  return {
    category: ELevelSpawnCategory.PROPS,
    clsid: EClsId.O_PHYS_S,
    index: 0,
    name: "crate",
    release: null,
    section: "physic_object",
    storyId: null,
    transform: mockVisualTransform({ x: 0, y: 0, z: 0 }),
    visual: 0,
    ...overrides,
  };
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
    engine: { engine: EXrayEngine.VANILLA, evidence: EXrayEngineEvidence.NAMED, subject: null },
    sun: null,
    hasSun: true,
    lights: 2,
    portals: 0,
    roots: createRoots([]),
    sectors: [mockSectorOutline()],
    start: null,
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

/**
 * Creates what a level view sets over the renderer's features: nothing, unless told.
 *
 * @param overrides - The groups the view sets.
 * @returns The view's feature options.
 */
export function mockLevelFeatureOptions(overrides: Partial<ILevelFeatureOptions> = {}): ILevelFeatureOptions {
  return {
    ambientOcclusion: {},
    antialiasing: null,
    grass: {},
    indirectLight: {},
    lights: {},
    reflections: {},
    shadows: {},
    water: {},
    ...overrides,
  };
}
