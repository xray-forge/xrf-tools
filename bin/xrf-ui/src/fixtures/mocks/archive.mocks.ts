import {
  ArchiveAnimationChannel,
  ArchiveAnmDescription,
  ArchiveChunksDescription,
  ArchiveDescribeScope,
  ArchiveDetailEntry,
  ArchiveDetailLibraryDescription,
  ArchiveDetailModel,
  ArchiveEfdDescription,
  ArchiveFileDescription,
  ArchiveFormatDescription,
  ArchiveLevelAiDescription,
  ArchiveLevelCollisionDescription,
  ArchiveLevelDescription,
  ArchiveLevelEnvModDescription,
  ArchiveLevelEnvModifier,
  ArchiveLevelFogVolDescription,
  ArchiveLevelGameDescription,
  ArchiveLevelHomDescription,
  ArchiveLevelLightsDescription,
  ArchiveLevelPsStaticDescription,
  ArchiveLevelSndStaticDescription,
  ArchiveLevelSndStaticSound,
  ArchiveLevelSomDescription,
  ArchiveLevelSurface,
  ArchiveLevelWallmarksDescription,
  ArchiveOmfDescription,
  ArchiveOmfMotion,
  ArchiveParticlesDescription,
  ArchiveParticlesEffect,
  ArchiveParticlesGroup,
  ArchivePpeColor,
  ArchivePpeDescription,
  ArchiveReference,
  ArchiveResolution,
  ArchiveShadersBlender,
  ArchiveShadersDescription,
  ArchiveShadowedCopy,
  ArchiveSpawnDescription,
  ArchiveSubject,
  ArchiveThmDescription,
  ArchiveWorld,
  ArchiveWorldEntry,
  EArchiveAnimationBehavior,
  EArchiveDescribeScope,
  EArchiveFormatDescription,
  EArchiveLevelEntry,
  EArchiveOmfTarget,
  EArchiveReferenceStatus,
  EArchiveSubject,
} from "@/core/ipc/types/xrf-app";
import {
  ArchiveDescriptor,
  ArchiveFileDescriptor,
  ArchiveProject,
  ArchiveReadPolicy,
  ArchiveSharedPayload,
} from "@/core/ipc/types/xrf-archive";
import { ArchiveStatistics } from "@/core/ipc/types/xrf-archive-stats";
import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { ArchivePackResult } from "@/core/ipc/types/xrf-pack";
import { EXrayAssetContainer, EXraySourceKind, XrayAssetContainer, XrayPathCollision } from "@/core/ipc/types/xrf-vfs";
import { Nullable } from "@/lib/types/general";

/**
 * Creates the complete result of a packing run.
 *
 * @param overrides - Result fields to override.
 * @returns A packing result with volume lists and timing metrics.
 */
export function mockArchivePackResult(overrides: Partial<ArchivePackResult> = {}): ArchivePackResult {
  return {
    outcome: "completed",
    volumes: ["C:\\out\\gamedata.db"],
    volumesOpened: ["C:\\out\\gamedata.db"],
    filesTotal: 1,
    filesSkipped: 0,
    filesStored: 1,
    filesCompressed: 0,
    filesAliased: 0,
    sizeSource: 100,
    sizeWritten: 120,
    duration: 1000,
    collectDuration: 100,
    writeDuration: 800,
    finalizeDuration: 100,
    speed: 100,
    ...overrides,
  };
}

/**
 * Creates an archive read policy fixture.
 *
 * @param overrides - Field values to override.
 * @returns An archive read policy fixture.
 */
export function mockArchiveReadPolicy(overrides: Partial<ArchiveReadPolicy> = {}): ArchiveReadPolicy {
  return {
    extensions: [
      EXrayExtension.BAT,
      EXrayExtension.CMD,
      EXrayExtension.CS,
      EXrayExtension.DS,
      EXrayExtension.GS,
      EXrayExtension.H,
      EXrayExtension.HLSL,
      EXrayExtension.HS,
      EXrayExtension.INI,
      EXrayExtension.JSON,
      EXrayExtension.LOG,
      EXrayExtension.LTX,
      EXrayExtension.LUA,
      EXrayExtension.MD,
      EXrayExtension.PS,
      EXrayExtension.PY,
      EXrayExtension.S,
      EXrayExtension.SCRIPT,
      EXrayExtension.SEQ,
      EXrayExtension.VS,
      EXrayExtension.XML,
    ],
    maximumSize: 10 * 1024 * 1024,
    imageExtensions: [EXrayExtension.DDS],
    maximumImageSize: 32 * 1024 * 1024,
    audioExtensions: [EXrayExtension.OGG],
    maximumAudioSize: 64 * 1024 * 1024,
    maximumDescribeSize: 64 * 1024 * 1024,
    maximumChunkTreeSize: 8 * 1024 * 1024,
    ...overrides,
  };
}

/**
 * Creates an archive file descriptor fixture.
 *
 * @param overrides - Field values to override.
 * @returns An archive file descriptor fixture.
 */
export function mockArchiveFileDescriptor(overrides: Partial<ArchiveFileDescriptor> = {}): ArchiveFileDescriptor {
  const descriptor: ArchiveFileDescriptor = {
    crc: 0x12345678,
    isDirectory: false,
    name: "configs\\system.ltx",
    offset: 4096,
    sizeCompressed: 2048,
    sizeReal: 2048,
    volume: 0,
    ...overrides,
  };

  // Derived exactly as `ArchiveFileDescriptor::new` derives it, so a fixture cannot describe an entry the reader would
  // never produce - a payload-less entry that is somehow still a file.
  return {
    ...descriptor,
    isDirectory: overrides.isDirectory ?? (!descriptor.sizeReal || /[\\/]$/.test(descriptor.name)),
  };
}

/**
 * Creates a shared payload fixture: the bytes one entry reads, read by other names too.
 *
 * Built from the entry's own location the way the backend derives it, so the fixture cannot describe a group its entry
 * would not be found in.
 *
 * @param descriptor - One entry read from the payload.
 * @param others - The other names read from it.
 * @returns A shared payload fixture with every name in sorted order.
 */
export function mockArchiveSharedPayload(
  descriptor: ArchiveFileDescriptor,
  others: Array<string>
): ArchiveSharedPayload {
  return {
    crc: descriptor.crc,
    names: [descriptor.name, ...others].sort(),
    offset: descriptor.offset,
    sizeCompressed: descriptor.sizeCompressed,
    sizeReal: descriptor.sizeReal,
    volume: descriptor.volume,
  };
}

/**
 * Creates an archives project fixture.
 *
 * @param files - File descriptors to include, or the default fixtures when omitted.
 * @returns An archives project fixture.
 */
export function mockArchivesProject(files?: Array<ArchiveFileDescriptor>): ArchiveProject {
  const descriptors: Array<ArchiveFileDescriptor> = files ?? [
    mockArchiveFileDescriptor(),
    mockArchiveFileDescriptor({
      name: "scripts\\actor.script",
      sizeReal: 1024,
      sizeCompressed: 1024,
    }),
  ];
  const archive: ArchiveDescriptor = {
    createdAt: null,
    entries: 0,
    modifiedAt: null,
    outputRootPath: "gamedata",
    path: "C:\\game\\database\\configs.db0",
    sizeCompressed: 0,
    sizeReal: 0,
  };

  return {
    archives: [archive],
    files: Object.fromEntries(descriptors.map((descriptor) => [descriptor.name, descriptor])),
    readPolicy: mockArchiveReadPolicy(),
    root: "C:\\game\\database",
    shadowed: [],
    sizeReal: descriptors.reduce((total: number, descriptor) => total + descriptor.sizeReal, 0),
  };
}

/**
 * Creates a path collision fixture: two entries folding onto one engine identity.
 *
 * @param overrides - Field values to override.
 * @returns A path collision fixture.
 */
export function mockPathCollision(overrides: Partial<XrayPathCollision> = {}): XrayPathCollision {
  return {
    kept: "C:/game/database/configs.db0::textures/a.dds",
    logicalPath: "textures\\a.dds",
    unreachable: "C:/game/database/patch.db0::Textures/A.DDS",
    ...overrides,
  };
}

/**
 * Creates a mounted-world entry fixture: one engine path, where it is read from, and what it hides.
 *
 * @param overrides - Field values to override.
 * @returns A world entry fixture.
 */
export function mockArchiveWorldEntry(overrides: Partial<ArchiveWorldEntry> = {}): ArchiveWorldEntry {
  return {
    container: mockLooseContainer(),
    name: "configs\\system.ltx",
    shadowed: [],
    sizeReal: 2048,
    ...overrides,
  };
}

/**
 * Creates a loose-file container fixture.
 *
 * @param relativePath - Path below the root, defaulting to the shared config fixture.
 * @param root - Tree the file sits in.
 * @returns A container naming a file on disk.
 */
export function mockLooseContainer(
  relativePath: string = "configs\\system.ltx",
  root: string = "C:\\game\\gamedata"
): XrayAssetContainer {
  return { kind: EXrayAssetContainer.DIRECTORY, relativePath, root };
}

/**
 * Creates an archived-entry container fixture.
 *
 * @param path - Volume holding the entry.
 * @returns A container naming an entry of a volume.
 */
export function mockArchivedContainer(path: string = "C:\\game\\db\\configs.db0"): XrayAssetContainer {
  return { kind: EXrayAssetContainer.ARCHIVE, path };
}

/**
 * Creates a hidden-copy fixture: where a shadowed copy sits, and how large that copy is.
 *
 * @param container - Where the hidden copy sits, defaulting to an archived entry.
 * @param sizeReal - Unpacked bytes of the hidden copy.
 * @returns A copy no lookup reaches.
 */
export function mockArchiveShadowedCopy(
  container: XrayAssetContainer = mockArchivedContainer(),
  sizeReal: number = 4096
): ArchiveShadowedCopy {
  return { container, sizeReal };
}

/**
 * Creates a mounted-world fixture: a game folder listed as the engine would resolve it.
 *
 * @param files - Entries to include, or one shadowing entry when omitted.
 * @returns A world fixture.
 */
export function mockArchivesWorld(files?: Array<ArchiveWorldEntry>): ArchiveWorld {
  const entries: Array<ArchiveWorldEntry> = files ?? [
    mockArchiveWorldEntry({ shadowed: [mockArchiveShadowedCopy()] }),
    mockArchiveWorldEntry({
      container: mockArchivedContainer(),
      name: "scripts\\actor.script",
      sizeReal: 1024,
    }),
  ];

  return {
    files: entries,
    mounts: ["C:\\game\\gamedata", "C:\\game\\db"],
    readPolicy: mockArchiveReadPolicy(),
    roots: { asset: null, roots: [{ mode: "auto", path: "C:\\game" }] },
    shadowedCount: entries.filter((entry: ArchiveWorldEntry) => entry.shadowed.length).length,
    shadowedSizeReal: entries.reduce(
      (total: number, entry: ArchiveWorldEntry) =>
        total + entry.shadowed.reduce((hidden: number, copy: ArchiveShadowedCopy) => hidden + copy.sizeReal, 0),
      0
    ),
    sizeReal: entries.reduce((total: number, entry: ArchiveWorldEntry) => total + entry.sizeReal, 0),
  };
}

/**
 * Creates the subject fixture a volume set opens as.
 *
 * @param files - File descriptors to include, or the default fixtures when omitted.
 * @returns What the explorer has open when it indexed volumes.
 */
export function mockArchivesVolumes(files?: Array<ArchiveFileDescriptor>): ArchiveSubject {
  return { kind: EArchiveSubject.VOLUMES, project: mockArchivesProject(files) };
}

/**
 * Creates the subject fixture a game folder opens as.
 *
 * @param files - Entries to include, or the default fixtures when omitted.
 * @returns What the explorer has open when it mounted a world.
 */
export function mockArchivesWorldSubject(files?: Array<ArchiveWorldEntry>): ArchiveSubject {
  return { kind: EArchiveSubject.WORLD, world: mockArchivesWorld(files) };
}

/**
 * @param overrides - Field values to override.
 * @returns A breakdown of a volume set.
 */
export function mockArchiveStatistics(overrides: Partial<ArchiveStatistics> = {}): ArchiveStatistics {
  return {
    compression: { sizeCompressed: 3072, sizeReal: 7168, storedUncompressed: 1 },
    extensions: [
      { extension: EXrayExtension.DDS, isDeclared: true, measure: { files: 1, sizeReal: 4096 }, sizeCompressed: 2048 },
      { extension: EXrayExtension.LTX, isDeclared: true, measure: { files: 4, sizeReal: 2048 }, sizeCompressed: 512 },
      { extension: "som", isDeclared: false, measure: { files: 2, sizeReal: 1024 }, sizeCompressed: 512 },
    ],
    folders: [
      { folder: "textures", measure: { files: 1, sizeReal: 4096 }, sizeCompressed: 2048 },
      { folder: "configs", measure: { files: 4, sizeReal: 2048 }, sizeCompressed: 512 },
      { folder: "meshes", measure: { files: 2, sizeReal: 1024 }, sizeCompressed: 512 },
    ],
    largest: [{ name: "textures\\wpn\\ak74.dds", sizeCompressed: 2048, sizeReal: 4096 }],
    origins: {
      archived: { files: 7, sizeReal: 7168 },
      hidden: { files: 0, sizeReal: 0 },
      loose: { files: 0, sizeReal: 0 },
      sources: [],
    },
    overview: {
      directories: 1,
      emptyFiles: 0,
      largestFile: 4096,
      meanFile: 1024,
      medianFile: 512,
      sources: 2,
      total: { files: 7, sizeReal: 7168 },
    },
    sizes: [
      { from: 1, measure: { files: 6, sizeReal: 3072 }, to: 1024 },
      { from: 1024, measure: { files: 1, sizeReal: 4096 }, to: 4096 },
    ],
    volumes: [
      {
        entries: 5,
        modifiedAt: null,
        path: "C:\\game\\db\\textures.db0",
        sizeCompressed: 2048,
        sizeReal: 4096,
      },
    ],
    ...overrides,
  };
}

/**
 * @param overrides - Field values to override.
 * @returns A breakdown of a world.
 */
export function mockArchiveWorldStatistics(overrides: Partial<ArchiveStatistics> = {}): ArchiveStatistics {
  return mockArchiveStatistics({
    compression: null,
    extensions: [
      { extension: EXrayExtension.LTX, isDeclared: true, measure: { files: 2, sizeReal: 3072 }, sizeCompressed: null },
    ],
    origins: {
      archived: { files: 1, sizeReal: 1024 },
      hidden: { files: 1, sizeReal: 8192 },
      loose: { files: 1, sizeReal: 2048 },
      sources: [
        {
          hides: { files: 0, sizeReal: 0 },
          isLoose: true,
          source: "C:\\game\\gamedata",
          wins: { files: 1, sizeReal: 2048 },
        },
        {
          hides: { files: 1, sizeReal: 8192 },
          isLoose: false,
          source: "C:\\game\\db\\configs.db0",
          wins: { files: 1, sizeReal: 1024 },
        },
      ],
    },
    volumes: null,
    ...overrides,
  });
}

/**
 * Creates the search a mounted world resolves through: a loose tree in front of the volumes it overrides.
 *
 * @param overrides - Field values to override.
 * @returns Where the open subject looks for an engine path, in the order it looks.
 */
export function mockArchiveResolution(overrides: Partial<ArchiveResolution> = {}): ArchiveResolution {
  return {
    sources: [
      {
        base: "",
        entries: 12,
        kind: EXraySourceKind.DIRECTORY,
        label: "gamedata",
        origin: "$game_data$",
        path: "C:\\game\\gamedata",
        step: "C:\\game",
        volumes: [],
      },
      {
        base: "",
        entries: 480,
        kind: EXraySourceKind.ARCHIVE,
        label: "db",
        origin: "$arch_dir$",
        path: "C:\\game\\db",
        step: "C:\\game",
        volumes: [
          {
            entries: 300,
            outputRootPath: "gamedata",
            path: "C:\\game\\db\\patch.db1",
            sizeCompressed: 1024,
            sizeReal: 4096,
          },
          {
            entries: 180,
            outputRootPath: "gamedata",
            path: "C:\\game\\db\\textures.db0",
            sizeCompressed: 2048,
            sizeReal: 8192,
          },
        ],
      },
    ],
    unread: [
      {
        origin: "$arch_dir_levels$",
        path: "C:\\game\\db\\levels",
        reason: "failed to read archive header",
      },
    ],
    ...overrides,
  };
}

/**
 * Creates a reference a description resolved against the subject being browsed.
 *
 * @param overrides - Field values to override.
 * @returns One file a description names, and what became of it.
 */
export function mockArchiveReference(overrides: Partial<ArchiveReference> = {}): ArchiveReference {
  return {
    name: "act\\act_arm_1",
    path: "textures\\act\\act_arm_1.dds",
    entry: "textures\\act\\act_arm_1.dds",
    status: EArchiveReferenceStatus.PRESENT,
    ...overrides,
  };
}

/**
 * Creates a texture descriptor description, with every chunk declared.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about one texture descriptor.
 */
export function mockArchiveThmDescription(overrides: Partial<ArchiveThmDescription> = {}): ArchiveThmDescription {
  return {
    texture: {
      reference: mockArchiveReference(),
      shape: { width: 512, height: 512, mipmapLevels: 10, format: "DXT5" },
      declared: null,
    },
    textureType: { label: "2D Texture", value: 0, isReadByEngine: true, isDeclared: true },
    bump: {
      modeLabel: "Use",
      mode: 2,
      virtualHeight: 0.05,
      texture: mockArchiveReference({
        name: "act\\act_arm_1_bump",
        path: "textures\\act\\act_arm_1_bump.dds",
        entry: null,
        status: EArchiveReferenceStatus.ABSENT,
      }),
      isUsed: true,
    },
    detail: { scale: 1, texture: null, enabledBy: [] },
    externalNormalMap: null,
    material: { label: "Blin <-> Phong", value: 1, weight: 0 },
    parameters: {
      formatLabel: "DXT5",
      format: 5,
      mipFilterLabel: "Box",
      mipFilter: 0,
      borderColor: 0,
      fadeColor: 0,
      fadeAmount: 0,
      width: 512,
      height: 512,
      flags: [
        { label: "flGenerateMipMaps", isSet: true },
        { label: "flDitherColor", isSet: false },
      ],
      unnamedFlags: 0,
    },
    fadeDelay: 0,
    file: { version: 0x0012, isSupportedVersion: true, thumbnailType: 1, thumbnail: null, extraChunks: [] },
    ...overrides,
  };
}

/**
 * Creates one motion of a bank, a cycle on no particular part playing at the rate it was sampled at.
 *
 * @param overrides - Field values to override.
 * @returns One motion, as a description carries it.
 */
export function mockArchiveOmfMotion(overrides: Partial<ArchiveOmfMotion> = {}): ArchiveOmfMotion {
  return {
    name: "norm_idle_0",
    frames: 46,
    durationSeconds: 46 / 30,
    playbackSeconds: 46 / 30,
    speed: { value: 1, declared: 1, isClamped: false },
    power: { value: 1, declared: 1, isClamped: false },
    blend: {
      accrue: 3,
      falloff: 2.99,
      declaredAccrue: 2,
      declaredFalloff: 2,
      isFalloffReplaced: true,
    },
    target: { kind: EArchiveOmfTarget.UNNAMED },
    flags: ["esmStopAtEnd"],
    unnamedFlags: 0,
    marks: [],
    hasDivergingLabel: false,
    ...overrides,
  };
}

/**
 * Creates one channel of an object motion, keyed the way a camera effect's position channel is.
 *
 * @param name - What the channel animates, which is the position it holds in the file.
 * @param overrides - Field values to override.
 * @returns One channel, as a description carries it.
 */
export function mockArchiveAnimationChannel(
  name: string = "position x",
  overrides: Partial<ArchiveAnimationChannel> = {}
): ArchiveAnimationChannel {
  return {
    name,
    keys: 9,
    firstSeconds: 0,
    lastSeconds: 2,
    minimum: -0.125,
    maximum: 0.25,
    behaviorBefore: { kind: EArchiveAnimationBehavior.CONSTANT },
    behaviorAfter: { kind: EArchiveAnimationBehavior.CONSTANT },
    shapes: ["tcb"],
    ...overrides,
  };
}

/**
 * Creates an object motion description: a two second camera effect keyed on all six channels.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about one object motion.
 */
export function mockArchiveAnmDescription(overrides: Partial<ArchiveAnmDescription> = {}): ArchiveAnmDescription {
  const channels: Array<ArchiveAnimationChannel> = overrides.channels ?? [
    mockArchiveAnimationChannel("position x"),
    mockArchiveAnimationChannel("position y"),
    mockArchiveAnimationChannel("position z", {
      keys: 0,
      firstSeconds: null,
      lastSeconds: null,
      minimum: null,
      maximum: null,
      shapes: [],
    }),
    mockArchiveAnimationChannel("rotation pitch", { shapes: ["tcb", "linear"] }),
    mockArchiveAnimationChannel("rotation heading"),
    mockArchiveAnimationChannel("rotation bank", { keys: 1, firstSeconds: 0, lastSeconds: 0, shapes: ["stepped"] }),
  ];

  return {
    name: null,
    version: 5,
    frameStart: 0,
    frameEnd: 59,
    frames: 60,
    fps: 30,
    durationSeconds: 2,
    keys: channels.reduce((total: number, channel: ArchiveAnimationChannel) => total + channel.keys, 0),
    keyedSeconds: 2,
    ...overrides,
    channels,
  };
}

/**
 * Creates one colour parameter of a post-process effect, keyed on red and green and flat on blue.
 *
 * @param name - Which colour it is, which is the position it holds in the file.
 * @param overrides - Field values to override.
 * @returns One colour parameter, as a description carries it.
 */
export function mockArchivePpeColor(
  name: string = "base color",
  overrides: Partial<ArchivePpeColor> = {}
): ArchivePpeColor {
  return {
    name,
    base: 0.5,
    keys: 4,
    lengthSeconds: 3,
    channels: [
      mockArchiveAnimationChannel("red", { keys: 2, firstSeconds: 0, lastSeconds: 3 }),
      mockArchiveAnimationChannel("green", { keys: 2, firstSeconds: 0, lastSeconds: 3 }),
      mockArchiveAnimationChannel("blue", {
        keys: 0,
        firstSeconds: null,
        lastSeconds: null,
        minimum: null,
        maximum: null,
        shapes: [],
      }),
    ],
    ...overrides,
  };
}

/**
 * Creates a post-process effect description: a version 2 effect grading through a gradient the subject holds.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about one post-process effect.
 */
export function mockArchivePpeDescription(overrides: Partial<ArchivePpeDescription> = {}): ArchivePpeDescription {
  const empty: Partial<ArchiveAnimationChannel> = {
    keys: 0,
    firstSeconds: null,
    lastSeconds: null,
    minimum: null,
    maximum: null,
    shapes: [],
  };

  return {
    version: 2,
    lengthSeconds: 5,
    keys: 11,
    colors: [
      mockArchivePpeColor("base color"),
      mockArchivePpeColor("add color", { keys: 0, lengthSeconds: 0 }),
      mockArchivePpeColor("gray color", { keys: 0, lengthSeconds: 0 }),
    ],
    values: [
      mockArchiveAnimationChannel("gray value", { keys: 3, firstSeconds: 0, lastSeconds: 5 }),
      mockArchiveAnimationChannel("blur", empty),
      mockArchiveAnimationChannel("duality horizontal", empty),
      mockArchiveAnimationChannel("duality vertical", empty),
      mockArchiveAnimationChannel("noise intensity", empty),
      mockArchiveAnimationChannel("noise granularity", empty),
      mockArchiveAnimationChannel("noise fps", empty),
    ],
    colorMap: {
      texture: mockArchiveReference({ name: "grad\\grad_psi" }),
      influence: mockArchiveAnimationChannel("colour map influence", { keys: 1, firstSeconds: 0, lastSeconds: 0 }),
      isUsed: true,
    },
    ...overrides,
  };
}

/**
 * Creates one detail object: a swaying mesh drawn with a texture the open subject holds.
 *
 * @param texture - Engine path of the texture it draws with, or null for an object naming none.
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about one detail object.
 */
export function mockArchiveDetailModel(
  texture: Nullable<string> = "detail\\detail_grass",
  overrides: Partial<ArchiveDetailModel> = {}
): ArchiveDetailModel {
  return {
    shader: "details\\blend",
    texture: texture ? mockArchiveReference({ name: texture, path: `textures\\${texture}.dds` }) : null,
    minScale: 0.5,
    maxScale: 1.5,
    isWaving: true,
    unnamedFlags: 0,
    vertices: 24,
    triangles: 12,
    bounds: { width: 0.35, height: 0.6, depth: 0.35 },
    ...overrides,
  };
}

/**
 * Creates one entry of a level's detail library.
 *
 * @param index - Position in the library, which is what a slot addresses.
 * @param overrides - Field values to override.
 * @returns One library entry, as a description carries it.
 */
export function mockArchiveDetailEntry(
  index: number = 0,
  overrides: Partial<ArchiveDetailEntry> = {}
): ArchiveDetailEntry {
  return {
    index,
    plantedCorners: 4820,
    model: mockArchiveDetailModel(),
    ...overrides,
  };
}

/**
 * Creates a detail layer description: a 351 by 351 grid planted across most of itself, from three objects.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a level's detail layer.
 */
export function mockArchiveDetailLibraryDescription(
  overrides: Partial<ArchiveDetailLibraryDescription> = {}
): ArchiveDetailLibraryDescription {
  return {
    version: 3,
    sizeX: 351,
    sizeZ: 351,
    slots: 123_201,
    plantedSlots: 87_842,
    coversX: 702,
    coversZ: 702,
    entries: [
      mockArchiveDetailEntry(0),
      mockArchiveDetailEntry(1, {
        plantedCorners: 0,
        model: mockArchiveDetailModel("detail\\detail_stone", {
          isWaving: false,
          bounds: null,
          vertices: 0,
          triangles: 0,
        }),
      }),
      mockArchiveDetailEntry(2, { plantedCorners: 1, model: mockArchiveDetailModel(null) }),
    ],
    ...overrides,
  };
}

/**
 * Creates a motion bank description: one partition of two parts, and two motions.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about one motion bank.
 */
export function mockArchiveOmfDescription(overrides: Partial<ArchiveOmfDescription> = {}): ArchiveOmfDescription {
  const motions: Array<ArchiveOmfMotion> = overrides.motions ?? [
    mockArchiveOmfMotion(),
    mockArchiveOmfMotion({ name: "norm_walk_0", frames: 60, durationSeconds: 2, playbackSeconds: 2 }),
  ];

  return {
    bank: {
      version: 4,
      carriesMarks: true,
      motions: motions.length,
      effects: 0,
      bones: 3,
      frames: 106,
      durationSeconds: 106 / 30,
      divergingLabels: 0,
      markedMotions: 0,
      replacedFalloffs: motions.length,
    },
    parts: [
      { name: "torso", bones: ["bip01_spine", "bip01_head"], cycles: 1 },
      { name: "legs", bones: ["bip01_l_thigh"], cycles: 1 },
    ],
    ...overrides,
    motions,
  };
}

/**
 * Creates the container of a file nothing reads: two chunks, the first holding two of its own.
 *
 * @param overrides - Field values to override.
 * @returns The shape a description gives an unknown container.
 */
export function mockArchiveChunksDescription(
  overrides: Partial<ArchiveChunksDescription> = {}
): ArchiveChunksDescription {
  return {
    chunks: [
      {
        id: 0x1,
        size: 64,
        isCompressed: false,
        children: [
          { id: 0x10, size: 24, isCompressed: false, children: [] },
          { id: 0x11, size: 24, isCompressed: false, children: [] },
        ],
      },
      { id: 0x1100, size: 512, isCompressed: false, children: [] },
    ],
    nodes: 4,
    depth: 2,
    size: 600,
    ...overrides,
  };
}

/**
 * Creates a level collision mesh description of the size Anomaly's larger levels reach.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.cform`.
 */
export function mockArchiveLevelCollisionDescription(
  overrides: Partial<ArchiveLevelCollisionDescription> = {}
): ArchiveLevelCollisionDescription {
  return {
    version: 4,
    vertices: 420_690,
    faces: 812_004,
    bounds: { width: 512, height: 128, depth: 512 },
    size: 41_352_144,
    ...overrides,
  };
}

/**
 * Creates a level navigation grid description.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.ai`.
 */
export function mockArchiveLevelAiDescription(
  overrides: Partial<ArchiveLevelAiDescription> = {}
): ArchiveLevelAiDescription {
  return {
    version: 9,
    nodes: 1_204_331,
    nodeSize: 0.7,
    nodeHeight: 0.4,
    bounds: { width: 512, height: 128, depth: 512 },
    guid: "6a0f0a1e-0000-4000-8000-000000000002",
    size: 34_185_416,
    ...overrides,
  };
}

/**
 * Creates one drawn row of a level shader table, binding a texture the open subject holds.
 *
 * @param index - Position in the table, which is what a face addresses.
 * @param overrides - Entry values to override.
 * @returns One surface, as a description carries it.
 */
export function mockArchiveLevelSurface(
  index: number = 1,
  overrides: Partial<ArchiveLevelSurface> = {}
): ArchiveLevelSurface {
  return {
    index,
    entry: {
      kind: EArchiveLevelEntry.DRAWN,
      shader: { name: "default", status: EArchiveReferenceStatus.PRESENT },
      textures: [
        mockArchiveReference({
          name: "wall\\wall_beton",
          path: "textures\\wall\\wall_beton.dds",
          entry: "textures\\wall\\wall_beton.dds",
        }),
      ],
    },
    ...overrides,
  };
}

/**
 * Creates a level bundle description: the empty row every built level has, and one drawn surface.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a compiled level bundle.
 */
export function mockArchiveLevelDescription(overrides: Partial<ArchiveLevelDescription> = {}): ArchiveLevelDescription {
  const surfaces: Array<ArchiveLevelSurface> = overrides.surfaces ?? [
    { index: 0, entry: { kind: EArchiveLevelEntry.SKIPPED } },
    mockArchiveLevelSurface(),
  ];

  return {
    bundle: {
      xrlcVersion: 14,
      xrlcQuality: 2,
      surfaces: surfaces.length,
      shaders: 1,
      undefinedShaders: 0,
      textures: 1,
      absentTextures: 0,
      library: mockArchiveReference({ name: "shaders.xr", path: "shaders.xr", entry: "shaders.xr" }),
      hasShaderTable: true,
    },
    ...overrides,
    surfaces,
  };
}

/**
 * Creates one emitter of a particle library, drawing a texture the open subject holds.
 *
 * @param overrides - Field values to override.
 * @returns One effect, as a description carries it.
 */
export function mockArchiveParticlesEffect(overrides: Partial<ArchiveParticlesEffect> = {}): ArchiveParticlesEffect {
  return {
    name: "explosions\\smoke",
    maxParticles: 10,
    timeLimit: 0.25,
    shader: "particles\\add",
    texture: mockArchiveReference({
      name: "pfx\\pfx_smoke_a",
      path: "textures\\pfx\\pfx_smoke_a.dds",
      entry: "textures\\pfx\\pfx_smoke_a.dds",
    }),
    actions: ["Source", "KillOld"],
    actionsCount: 6,
    flags: 0,
    ...overrides,
  };
}

/**
 * Creates one sequence of a particle library, playing one effect the library defines.
 *
 * @param overrides - Field values to override.
 * @returns One group, as a description carries it.
 */
export function mockArchiveParticlesGroup(overrides: Partial<ArchiveParticlesGroup> = {}): ArchiveParticlesGroup {
  return {
    name: "explosions\\blast",
    timeLimit: 0,
    effects: [
      {
        effect: { name: "explosions\\smoke", isDefined: true },
        onBirth: null,
        onPlay: null,
        onDead: null,
        from: 0,
        to: 1,
        flags: 6,
      },
    ],
    ...overrides,
  };
}

/**
 * Creates a particle library description: one effect and one group that plays it.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about the particle library.
 */
export function mockArchiveParticlesDescription(
  overrides: Partial<ArchiveParticlesDescription> = {}
): ArchiveParticlesDescription {
  const effects: Array<ArchiveParticlesEffect> = overrides.effects ?? [mockArchiveParticlesEffect()];
  const groups: Array<ArchiveParticlesGroup> = overrides.groups ?? [mockArchiveParticlesGroup()];

  return {
    library: {
      version: 1,
      effects: effects.length,
      groups: groups.length,
      actions: 6,
      textures: 1,
      absentTextures: 0,
      undefinedEffects: 0,
    },
    ...overrides,
    effects,
    groups,
  };
}

/**
 * Creates one blender of a shader library, binding a texture and a slot the renderer fills.
 *
 * @param overrides - Field values to override.
 * @returns One blender, as a description carries it.
 */
export function mockArchiveShadersBlender(overrides: Partial<ArchiveShadersBlender> = {}): ArchiveShadersBlender {
  return {
    name: "models\\model_aref",
    class: "MODEL",
    version: 2,
    computer: "GSC-WS-14",
    time: 0,
    properties: [
      { name: "Name", kind: "Texture", value: "$base0", texture: null },
      {
        name: "R2-R",
        kind: "Texture",
        value: "detail\\detail_grnd_grass",
        texture: mockArchiveReference({
          name: "detail\\detail_grnd_grass",
          path: "textures\\detail\\detail_grnd_grass.dds",
          entry: "textures\\detail\\detail_grnd_grass.dds",
        }),
      },
      { name: "Priority", kind: "Integer", value: "4 (0 to 8)", texture: null },
    ],
    ...overrides,
  };
}

/**
 * Creates a shader library description holding one blender.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about the blender library.
 */
export function mockArchiveShadersDescription(
  overrides: Partial<ArchiveShadersDescription> = {}
): ArchiveShadersDescription {
  const blenders: Array<ArchiveShadersBlender> = overrides.blenders ?? [mockArchiveShadersBlender()];

  return {
    library: { blenders: blenders.length, classes: 1, textures: 1, absentTextures: 0 },
    ...overrides,
    blenders,
  };
}

/**
 * Creates a spawn set description whose game graph is most of the file, as vanilla's is.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a spawn set.
 */
export function mockArchiveSpawnDescription(overrides: Partial<ArchiveSpawnDescription> = {}): ArchiveSpawnDescription {
  return {
    version: 10,
    guid: "6a0f0a1e-0000-4000-8000-000000000001",
    graphGuid: "6a0f0a1e-0000-4000-8000-000000000002",
    objects: 6464,
    levels: 5,
    sections: [
      { id: 0, label: "Header", size: 44 },
      { id: 1, label: "ALife objects", size: 1_732_838 },
      { id: 4, label: "Game graph", size: 27_188_310 },
      { id: 9, label: null, size: 128 },
    ],
    size: 30_619_587,
    ...overrides,
  };
}

/**
 * Creates what the backend says about one entry the viewer cannot draw.
 *
 * @param format - The described format, defaulting to a texture descriptor.
 * @param scope - What the reference lookups searched, defaulting to a volume set.
 * @returns One described entry, and the scope behind it.
 */
export function mockArchiveFileDescription(
  format: ArchiveFormatDescription = {
    kind: EArchiveFormatDescription.THM,
    description: mockArchiveThmDescription(),
  },
  scope: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 }
): ArchiveFileDescription {
  return { scope, format };
}

/**
 * Creates a level occlusion mesh description of the size a mid-sized level reaches.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.hom`.
 */
export function mockArchiveLevelHomDescription(
  overrides: Partial<ArchiveLevelHomDescription> = {}
): ArchiveLevelHomDescription {
  return {
    version: 2,
    triangles: 1860,
    bounds: { width: 512, height: 128, depth: 512 },
    ...overrides,
  };
}

/**
 * Creates a level sound occlusion mesh description, half of whose faces occlude both ways.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.som`.
 */
export function mockArchiveLevelSomDescription(
  overrides: Partial<ArchiveLevelSomDescription> = {}
): ArchiveLevelSomDescription {
  return {
    version: 0,
    triangles: 218,
    twoSided: 96,
    faces: 314,
    minimumOcclusion: 0.1,
    maximumOcclusion: 0.85,
    bounds: { width: 256, height: 64, depth: 256 },
    ...overrides,
  };
}

/**
 * Creates one local weather override.
 *
 * @param overrides - Field values to override.
 * @returns One modifier, as a description carries it.
 */
export function mockArchiveLevelEnvModifier(overrides: Partial<ArchiveLevelEnvModifier> = {}): ArchiveLevelEnvModifier {
  return {
    radius: 40,
    power: 1,
    farPlane: 300,
    fogDensity: 0.85,
    usedParameters: ["fog colour", "fog density"],
    declaresParameters: true,
    ...overrides,
  };
}

/**
 * Creates a level weather override description holding one modifier.
 *
 * Most shipped levels carry none at all, which `modifiers: []` is the fixture for.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.env_mod`.
 */
export function mockArchiveLevelEnvModDescription(
  overrides: Partial<ArchiveLevelEnvModDescription> = {}
): ArchiveLevelEnvModDescription {
  return {
    version: 22,
    modifiers: [mockArchiveLevelEnvModifier()],
    ...overrides,
  };
}

/**
 * Creates a level particle placement description, whose few effects are planted many times over.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.ps_static`.
 */
export function mockArchiveLevelPsStaticDescription(
  overrides: Partial<ArchiveLevelPsStaticDescription> = {}
): ArchiveLevelPsStaticDescription {
  return {
    version: 1,
    placements: 129,
    restricted: 12,
    effects: [
      { name: "industrial\\steam_01", placements: 65, restricted: 0 },
      { name: "zones\\zone_acidic_idle", placements: 52, restricted: 12 },
      { name: "weather\\rain_splash", placements: 12, restricted: 0 },
    ],
    ...overrides,
  };
}

/**
 * Creates one sound a level plants.
 *
 * @param overrides - Field values to override.
 * @returns One planted sound, as a description carries it.
 */
export function mockArchiveLevelSndStaticSound(
  overrides: Partial<ArchiveLevelSndStaticSound> = {}
): ArchiveLevelSndStaticSound {
  return {
    sound: mockArchiveReference({
      name: "ambient\\day\\birds_1",
      path: "sounds\\ambient\\day\\birds_1.ogg",
      entry: "sounds\\ambient\\day\\birds_1.ogg",
    }),
    volume: 0.7,
    frequency: 1,
    isScheduled: false,
    activeFrom: 0,
    activeTo: 0,
    ...overrides,
  };
}

/**
 * Creates a level planted sound description, one of whose sounds only plays at night.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.snd_static`.
 */
export function mockArchiveLevelSndStaticDescription(
  overrides: Partial<ArchiveLevelSndStaticDescription> = {}
): ArchiveLevelSndStaticDescription {
  return {
    scheduled: 1,
    sounds: [
      mockArchiveLevelSndStaticSound(),
      mockArchiveLevelSndStaticSound({
        sound: mockArchiveReference({
          name: "ambient\\night\\owl_2",
          path: null,
          entry: null,
          status: EArchiveReferenceStatus.ABSENT,
        }),
        volume: 0.35,
        frequency: 1.25,
        isScheduled: true,
        activeFrom: 22,
        activeTo: 4,
      }),
      mockArchiveLevelSndStaticSound({ sound: null }),
    ],
    ...overrides,
  };
}

/**
 * Creates a level game data description of the size vanilla's larger levels reach.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.game`.
 */
export function mockArchiveLevelGameDescription(
  overrides: Partial<ArchiveLevelGameDescription> = {}
): ArchiveLevelGameDescription {
  return {
    spawns: [
      { label: "actor spawn", kind: 0, points: 4444, profiled: 0 },
      { label: "artefact spawn", kind: 1, points: 334, profiled: 0 },
      { label: "item spawn", kind: 2, points: 173, profiled: 173 },
    ],
    rpoints: 4951,
    ways: 1510,
    wayPoints: 8204,
    emptyWays: 3,
    ...overrides,
  };
}

/**
 * Creates a level volumetric fog description holding one body.
 *
 * Every shipped level but one carries no body at all, which `volumes: []` is the fixture for.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.fog_vol`.
 */
export function mockArchiveLevelFogVolDescription(
  overrides: Partial<ArchiveLevelFogVolDescription> = {}
): ArchiveLevelFogVolDescription {
  return {
    version: 1,
    obstacles: 144,
    volumes: [
      {
        profile: mockArchiveReference({
          name: "fog_vol_default",
          path: "configs\\environment\\fog_vol_default.ltx",
          entry: "configs\\environment\\fog_vol_default.ltx",
        }),
        obstacles: 144,
      },
    ],
    ...overrides,
  };
}

/**
 * Creates a compiled light list description, whose header chunk is the only one the runtime opens.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `build.lights`.
 */
export function mockArchiveLevelLightsDescription(
  overrides: Partial<ArchiveLevelLightsDescription> = {}
): ArchiveLevelLightsDescription {
  return {
    lights: 2203,
    used: 918,
    groups: [
      { id: 1, lights: 926, point: 918, isReadByEngine: true, isLights: true },
      { id: 2, lights: 1277, point: 1180, isReadByEngine: false, isLights: true },
      { id: 3, lights: 0, point: 0, isReadByEngine: false, isLights: false },
    ],
    bounds: { width: 512, height: 128, depth: 512 },
    ...overrides,
  };
}

/**
 * Creates a baked decal description, one of whose slots names no texture.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about a `level.wallmarks`.
 */
export function mockArchiveLevelWallmarksDescription(
  overrides: Partial<ArchiveLevelWallmarksDescription> = {}
): ArchiveLevelWallmarksDescription {
  return {
    marks: 441,
    vertices: 5292,
    slots: [
      {
        shader: "effects\\wallmark",
        texture: mockArchiveReference({
          name: "wm\\wm_blood",
          path: "textures\\wm\\wm_blood.dds",
          entry: "textures\\wm\\wm_blood.dds",
        }),
        marks: 347,
        vertices: 4164,
      },
      { shader: "effects\\wallmark", texture: null, marks: 94, vertices: 1128 },
    ],
    ...overrides,
  };
}

/**
 * Creates a trained evaluation function description of the shape every shipped one has.
 *
 * Every `.efd` in the workspace trees carries exactly one term over one to four inputs.
 *
 * @param overrides - Field values to override.
 * @returns Everything the viewer says about an `.efd`.
 */
export function mockArchiveEfdDescription(overrides: Partial<ArchiveEfdDescription> = {}): ArchiveEfdDescription {
  return {
    builderVersion: 3,
    dataFormat: 1,
    functionType: 70,
    minimumResult: 0,
    maximumResult: 1,
    variableRanges: [10, 4],
    variableKinds: [21, 46],
    patterns: [{ variables: [0, 1], weights: 40 }],
    weights: 40,
    ...overrides,
  };
}
