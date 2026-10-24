import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import {
  ELevelSpawnCategory,
  LevelEntry,
  LevelMapDescription,
  LevelSpawnObject,
  LevelStart,
  LevelTextureReference,
  SelectedLevelDescription,
} from "@/core/ipc/types/xrf-app";
import { EXrayEngine, EXrayEngineEvidence } from "@/core/ipc/types/xrf-engine-target";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ERenderTextureGroup } from "@/core/ipc/types/xrf-renderer";
import { EClsId } from "@/core/ipc/types/xrf-spawn";
import { SectorOutline } from "@/core/ipc/types/xrf-visual";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { InvokeHandler, InvokeMap } from "@/fixtures/mocks/tauri.mocks";
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
    shaderEntries: 4,
    source: { kind: "directory", path: "C:\\levels\\zaton" },
    visuals: 2,
    xrlcQuality: 1,
    xrlcVersion: 14,
    ...overrides,
  };
}

/** The parts of an open level a fixture answers, beside the level itself; each missing one answers as empty. */
export interface IMockLevelParts {
  levelMap?: Nullable<LevelMapDescription>;
  start?: Nullable<LevelStart>;
  surfaces?: Array<XraySurfaceDescriptor>;
  textures?: Array<LevelTextureReference>;
}

/**
 * What the open level's part queries answer, each a snapshot of the opening asked about.
 *
 * @param parts - The parts to answer with.
 * @returns Responses for the `describe_*` queries of a level's parts.
 */
export function mockLevelPartResponses(parts: IMockLevelParts = {}): InvokeMap {
  return {
    ["plugin:levels|describe_level_map"]: mockLevelPartAnswer(parts.levelMap ?? null),
    ["plugin:levels|describe_start"]: mockLevelPartAnswer(parts.start ?? null),
    ["plugin:levels|describe_surfaces"]: mockLevelPartAnswer(parts.surfaces ?? []),
    ["plugin:levels|describe_textures"]: mockLevelPartAnswer(parts.textures ?? []),
  };
}

/**
 * @param value - What a part query answers, null included.
 * @returns A query answering it as a snapshot of the opening asked about.
 */
export function mockLevelPartAnswer(value: unknown): InvokeHandler {
  return (args?: Record<string, unknown>) => mockSessionSnapshot(value, args?.sessionId as string);
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
      .map(([reference, loaded]) => ({
        group: ERenderTextureGroup.SURFACES,
        reason: loaded.reason as string,
        reference,
      })),
    deferred: 0,
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
    lightModel: null,
    displayTransform: null,
    lightLevel: null,
    sunTint: null,
    upscaling: {},
    output: {},
    debanding: {},
    enhancedBloom: {},
    fog: {},
    glass: {},
    grading: {},
    grass: {},
    indirectLight: {},
    lights: {},
    parallax: {},
    rain: {},
    reflections: {},
    shadows: {},
    volumetric: {},
    water: {},
    ...overrides,
  };
}
