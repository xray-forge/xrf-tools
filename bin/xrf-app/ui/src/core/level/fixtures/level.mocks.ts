import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { IpcCommandMocks } from "@xrf/ipc/testing";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import {
  ELevelObjectId,
  ELevelObjectKind,
  ELevelObjectShape,
  ELevelRestriction,
  LevelMapDescription,
  LevelObject,
  LevelObjectSpawn,
  LevelStart,
  LevelTextureReference,
  SelectedLevelDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngine, EXrayEngineEvidence } from "@xrf/ipc/types/xrf-engine-target";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { ERenderTextureGroup } from "@xrf/ipc/types/xrf-renderer";
import { EClsId } from "@xrf/ipc/types/xrf-spawn";
import { SectorOutline } from "@xrf/ipc/types/xrf-visual";
import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { mockVisualBounds, mockVisualTransform } from "@/core/visuals/fixtures/visual.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";

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
 * One level object the spawn places and the viewer draws: a crate at the origin, the spawn's first object, standing as
 * the first visual.
 *
 * @param overrides - Fields to replace on the object.
 * @param spawn - Fields to replace on what the spawn records of it.
 * @returns An object fixture.
 */
export function mockLevelObject(
  overrides: Partial<LevelObject> = {},
  spawn: Partial<LevelObjectSpawn> = {}
): LevelObject {
  return {
    id: { index: 0, source: ELevelObjectId.SPAWN },
    kind: ELevelObjectKind.PROPS,
    name: "crate",
    spawn: {
      clsid: EClsId.O_PHYS_S,
      release: null,
      section: "physic_object",
      storyId: null,
      visual: 0,
      ...spawn,
    },
    transform: mockVisualTransform({ x: 0, y: 0, z: 0 }),
    volume: null,
    ...overrides,
  };
}

/**
 * A space restrictor keeping no one: a sphere of `radius` metres about its place.
 *
 * @param index - Its index among the spawn's objects.
 * @param radius - Its sphere's radius.
 * @returns An object fixture drawn as nothing.
 */
export function mockLevelVolumeObject(index: number, radius: number = 2): LevelObject {
  return mockLevelObject(
    {
      id: { index, source: ELevelObjectId.SPAWN },
      kind: ELevelObjectKind.RESTRICTORS,
      name: `restrictor_${index}`,
      volume: {
        destination: null,
        restriction: ELevelRestriction.NONE,
        shapes: [{ center: { x: 0, y: 0, z: 0 }, kind: ELevelObjectShape.SPHERE, radius }],
      },
    },
    { clsid: EClsId.SPC_RS_S, section: "space_restrictor", visual: null }
  );
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
export function mockLevelPartResponses(parts: IMockLevelParts = {}): IpcCommandMocks {
  return {
    [ELevelsCommand.DESCRIBE_LEVEL_MAP]: mockLevelPartAnswer(parts.levelMap ?? null),
    [ELevelsCommand.DESCRIBE_START]: mockLevelPartAnswer(parts.start ?? null),
    [ELevelsCommand.DESCRIBE_SURFACES]: mockLevelPartAnswer(parts.surfaces ?? []),
    [ELevelsCommand.DESCRIBE_TEXTURES]: mockLevelPartAnswer(parts.textures ?? []),
  };
}

/**
 * @param value - What a part query answers, null included.
 * @param revision - The revision it is read at.
 * @returns A query answering it as a snapshot of the opening asked about.
 */
export function mockLevelPartAnswer<T>(
  value: T,
  revision: number = 0
): (args: { sessionId: string }) => SessionSnapshot<T> {
  return (args: { sessionId: string }) => mockSessionSnapshot(value, args.sessionId, revision);
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
    camera: {},
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
    terrain: {},
    volumetric: {},
    water: {},
    ...overrides,
  };
}
