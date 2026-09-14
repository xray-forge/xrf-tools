import {
  ArchiveResolution,
  ArchiveShadowedCopy,
  ArchiveSubject,
  ArchiveWorld,
  ArchiveWorldEntry,
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
import { EXraySourceKind, XrayAssetContainer, XrayPathCollision } from "@/core/ipc/types/xrf-vfs";

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
      EXrayExtension.CMD,
      EXrayExtension.DS,
      EXrayExtension.H,
      EXrayExtension.HS,
      EXrayExtension.JSON,
      EXrayExtension.LTX,
      EXrayExtension.MD,
      EXrayExtension.PS,
      EXrayExtension.S,
      EXrayExtension.SCRIPT,
      EXrayExtension.VS,
      EXrayExtension.XML,
    ],
    maximumSize: 10 * 1024 * 1024,
    imageExtensions: [EXrayExtension.DDS],
    maximumImageSize: 32 * 1024 * 1024,
    audioExtensions: [EXrayExtension.OGG],
    maximumAudioSize: 64 * 1024 * 1024,
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
  return { kind: "directory", relativePath, root };
}

/**
 * Creates an archived-entry container fixture.
 *
 * @param path - Volume holding the entry.
 * @returns A container naming an entry of a volume.
 */
export function mockArchivedContainer(path: string = "C:\\game\\db\\configs.db0"): XrayAssetContainer {
  return { kind: "archive", path };
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
  return { kind: "volumes", project: mockArchivesProject(files) };
}

/**
 * Creates the subject fixture a game folder opens as.
 *
 * @param files - Entries to include, or the default fixtures when omitted.
 * @returns What the explorer has open when it mounted a world.
 */
export function mockArchivesWorldSubject(files?: Array<ArchiveWorldEntry>): ArchiveSubject {
  return { kind: "world", world: mockArchivesWorld(files) };
}

/**
 * @param overrides - Field values to override.
 * @returns A breakdown of a volume set.
 */
export function mockArchiveStatistics(overrides: Partial<ArchiveStatistics> = {}): ArchiveStatistics {
  return {
    compression: { sizeCompressed: 3072, sizeReal: 7168, storedUncompressed: 1 },
    extensions: [
      { extension: "dds", isDeclared: true, measure: { files: 1, sizeReal: 4096 }, sizeCompressed: 2048 },
      { extension: "ltx", isDeclared: true, measure: { files: 4, sizeReal: 2048 }, sizeCompressed: 512 },
      { extension: "som", isDeclared: false, measure: { files: 2, sizeReal: 1024 }, sizeCompressed: 512 },
    ],
    folders: [
      { folder: "textures", measure: { files: 1, sizeReal: 4096 }, sizeCompressed: 2048 },
      { folder: "configs", measure: { files: 4, sizeReal: 2048 }, sizeCompressed: 512 },
      { folder: "meshes", measure: { files: 2, sizeReal: 1024 }, sizeCompressed: 512 },
    ],
    largest: [{ name: "textures\\wpn\\ak74.dds", sizeCompressed: 2048, sizeReal: 4096 }],
    origins: null,
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
    extensions: [{ extension: "ltx", isDeclared: true, measure: { files: 2, sizeReal: 3072 }, sizeCompressed: null }],
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
