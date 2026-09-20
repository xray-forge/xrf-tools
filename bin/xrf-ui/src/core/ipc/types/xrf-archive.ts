// Auto-generated rust bindings. Do not edit it manually.

import { XrayExtension } from "@/core/ipc/types/xrf-extension";

/** One volume of a set: where it is, where it mounts, and what it holds, counted at read time. */
export type ArchiveDescriptor = {
  /** Volume file creation time in Unix milliseconds, when the filesystem reports one. */
  createdAt: number | null;
  /** Volume file modification time in Unix milliseconds, when the filesystem reports one. */
  modifiedAt: number | null;
  /** Entries this volume's name table holds, before any merge shadows one of them. */
  entries: number;
  /** Root the volume unpacks under, from `[header] entry_point` with its alias stripped. */
  outputRootPath: string;
  /** The volume file this descriptor was read from. */
  path: string;
  /** Bytes this volume's entries occupy as stored, summed while its name table was read. */
  sizeCompressed: number;
  /** Bytes this volume's entries occupy once unpacked, summed while its name table was read. */
  sizeReal: number;
};

/** One entry of a volume's name table: where its payload sits and how to verify it. */
export type ArchiveFileDescriptor = {
  /** CRC32 of the unpacked payload, recorded by the packer and verified on decompression. */
  crc: number;
  /** Whether the entry names a directory rather than a file with bytes. */
  isDirectory: boolean;
  /** Entry name as authored, which the engine registers verbatim. */
  name: string;
  /** Byte offset of the payload inside its volume. */
  offset: number;
  /** Payload bytes as stored in the volume. */
  sizeCompressed: number;
  /** Payload bytes once unpacked. */
  sizeReal: number;
  /** Which volume holds the payload, as a position in [`crate::ArchiveProject::archives`]. */
  volume: number;
};

/** One volume set at a path the caller names, merged into a single name table. */
export type ArchiveProject = {
  /**
   * Volumes in merge order: a later one wins the name table, so a caller searching them as separate sources must
   * search them in reverse to resolve an entry to the bytes this project's table names.
   */
  archives: Array<ArchiveDescriptor>;
  /** Entries keyed by their authored name, which is the same allocation each descriptor carries as its `name`. */
  files: { [key in string]: ArchiveFileDescriptor };
  /** Entries a later volume overwrote in the merge, in the order they were displaced. */
  shadowed: Array<ArchiveFileDescriptor>;
  readPolicy: ArchiveReadPolicy;
  /**
   * The tightest path holding exactly these volumes: the volume itself when one file was read, the volumes' common
   * parent when a directory was walked. Mounting it reaches this project's entries and no others, which is what a
   * caller reading an entry's bytes back out of the filesystem needs.
   */
  root: string;
  sizeReal: number;
};

/** What a viewer may read out of a mounted tree, by extension and size. */
export type ArchiveReadPolicy = {
  extensions: Array<XrayExtension>;
  maximumSize: number;
  /** Extensions decoded into a picture. Compression does not apply: it is undone before decoding. */
  textureExtensions: Array<XrayExtension>;
  maximumTextureSize: number;
  /** Extensions the webview renders itself, so the backend hands the bytes over rather than decoding them. */
  imageExtensions: Array<XrayExtension>;
  maximumImageSize: number;
  /** Extensions played by the webview itself, so the backend only has to hand over the bytes. */
  audioExtensions: Array<XrayExtension>;
  maximumAudioSize: number;
  /** Ceiling on an entry read whole to describe its format. */
  maximumDescribeSize: number;
  /** Ceiling on an entry read whole only to walk the container it is, which buys far less and so admits far less. */
  maximumChunkTreeSize: number;
};

/** One text file read for display: its name, decoded content, and unpacked size. */
export type ArchiveReadResult = {
  /** Name the content was read under. */
  name: string;
  /** Text decoded from Windows-1251, like every engine text format. */
  content: string;
  /** Bytes once unpacked, before decoding. */
  size: number;
};

/** Stored bytes that several file entries of one volume set locate at once. */
export type ArchiveSharedPayload = {
  /** Which volume holds the bytes, as a position in [`crate::ArchiveProject::archives`]. */
  volume: number;
  /** Byte offset of the payload inside its volume. */
  offset: number;
  /** Payload bytes as stored in the volume. */
  sizeCompressed: number;
  /** Payload bytes once unpacked. */
  sizeReal: number;
  /** CRC32 of the unpacked payload. */
  crc: number;
  /** Authored names of every file entry located here, in name order; always two or more. */
  names: Array<string>;
};
