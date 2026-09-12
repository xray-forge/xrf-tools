import { ArchiveSubject, ArchiveWorld, ArchiveWorldEntry } from "@/core/bindings/types/xrf-app";
import {
  ArchiveDescriptor,
  ArchiveFileDescriptor,
  ArchiveProject,
  ArchiveReadPolicy,
  ArchiveSharedPayload,
} from "@/core/bindings/types/xrf-archive";
import { XrayAssetContainer, XrayPathCollision } from "@/core/bindings/types/xrf-vfs";

/**
 * Creates an archive read policy fixture.
 *
 * @param overrides - Field values to override.
 * @returns An archive read policy fixture.
 */
export function mockArchiveReadPolicy(overrides: Partial<ArchiveReadPolicy> = {}): ArchiveReadPolicy {
  return {
    extensions: ["cmd", "ds", "h", "hs", "json", "ltx", "md", "ps", "s", "script", "vs", "xml"],
    maximumSize: 10 * 1024 * 1024,
    imageExtensions: ["dds"],
    maximumImageSize: 32 * 1024 * 1024,
    audioExtensions: ["ogg"],
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
 * Creates a mounted-world fixture: a game folder listed as the engine would resolve it.
 *
 * @param files - Entries to include, or one shadowing entry when omitted.
 * @returns A world fixture.
 */
export function mockArchivesWorld(files?: Array<ArchiveWorldEntry>): ArchiveWorld {
  const entries: Array<ArchiveWorldEntry> = files ?? [
    mockArchiveWorldEntry({ shadowed: [mockArchivedContainer()] }),
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
