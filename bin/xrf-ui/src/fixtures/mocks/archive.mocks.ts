import {
  ArchiveDescriptor,
  ArchiveFileDescriptor,
  ArchiveProject,
  ArchiveProjectReadPolicy,
} from "@/core/bindings/types/xrf-archive";
import { XrayPathCollision } from "@/core/bindings/types/xrf-vfs";

/**
 * Creates an archive read policy fixture.
 *
 * @param overrides - Field values to override.
 * @returns An archive read policy fixture.
 */
export function mockArchiveReadPolicy(overrides: Partial<ArchiveProjectReadPolicy> = {}): ArchiveProjectReadPolicy {
  return {
    extensions: ["ltx", "script", "ps", "ds", "h", "hs", "s", "vs", "cmd", "xml"],
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
