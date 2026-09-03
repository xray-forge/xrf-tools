// Auto-generated rust bindings. Do not edit it manually.

/**
 * One volume of a set: where it is, where it mounts, and what it holds, counted at read time.
 *
 * The entries themselves live in [`crate::ArchiveProject::files`] and nowhere else. Retaining a per-volume copy cost
 * one full duplicate of every descriptor in the set, and the only thing that ever read it back was these three
 * totals.
 */
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

/**
 * One entry of a volume's name table: where its payload sits and how to verify it.
 *
 * Equal `size_real` and `size_compressed` is how the format says "stored uncompressed".
 */
export type ArchiveFileDescriptor = {
  /** CRC32 of the unpacked payload, recorded by the packer and verified on decompression. */
  crc: number;
  /**
   * Whether the entry names a directory rather than a file with bytes.
   *
   * A volume records the directories it contains so an unpacker can recreate them. X-Ray marks those entries with a
   * trailing separator; a zero-length entry without one is an empty file.
   */
  isDirectory: boolean;
  /**
   * Entry name as authored, which the engine registers verbatim.
   *
   * Shared rather than owned outright: the merged name table keys entries by this same name and a mounted archive
   * maps its engine identity back to it, so an owned copy per structure meant three allocations for one name.
   */
  name: string;
  /** Byte offset of the payload inside its volume. */
  offset: number;
  /** Payload bytes as stored in the volume. */
  sizeCompressed: number;
  /** Payload bytes once unpacked. */
  sizeReal: number;
  /**
   * Which volume holds the payload, as a position in [`crate::ArchiveProject::archives`].
   *
   * A position rather than a path, because an entry belongs to a project and the project already describes each
   * volume once. Naming the volume again per entry would make every read a search for it, and would let an entry
   * claim a volume its own project does not hold. The position is also the volume's merge rank, which is what
   * decides between two entries claiming one name.
   *
   * Set by [`crate::ArchiveProject`] as it merges each volume, and stable for the life of that project.
   */
  volume: number;
};

/**
 * One volume set at a path the caller names, merged into a single name table.
 *
 * Scoped to a path on purpose: which directories of an installation hold volumes is a question the mount planner in
 * `xrf-vfs` answers (`XrayMountPlan::from_fsgame`), and answering it here too would put `fsgame.ltx` knowledge in the
 * volume-format layer and give the same declaration two readers.
 *
 * Later volumes win the merge, so a patch volume shadows the entry it replaces.
 */
export type ArchiveProject = {
  /**
   * Volumes in merge order: a later one wins the name table, so a caller searching them as separate sources must
   * search them in reverse to resolve an entry to the bytes this project's table names.
   */
  archives: Array<ArchiveDescriptor>;
  /** Entries keyed by their authored name, which is the same allocation each descriptor carries as its `name`. */
  files: { [key in string]: ArchiveFileDescriptor };
  readPolicy: ArchiveProjectReadPolicy;
  /**
   * The tightest path holding exactly these volumes: the volume itself when one file was read, the volumes' common
   * parent when a directory was walked. Mounting it reaches this project's entries and no others, which is what a
   * caller reading an entry's bytes back out of the filesystem needs.
   */
  root: string;
  sizeReal: number;
};

/**
 * What an archive viewer may read out of a project, by extension and size.
 *
 * A gate for interactive consumers rather than a format rule: [`crate::ArchiveProject::read_file_bytes`] ignores it,
 * while [`crate::ArchiveProject::read_file_as_string`] refuses what the policy does not cover.
 *
 * Only the text lists are enforced here. The picture and sound lists are routing hints for the viewer, which reads
 * both through the shared mounted assets rather than through this project, and so answers to no limit of its own.
 */
export type ArchiveProjectReadPolicy = {
  extensions: Array<string>;
  maximumSize: number;
  /** Extensions decoded into a picture. Compression does not apply: it is undone before decoding. */
  imageExtensions: Array<string>;
  maximumImageSize: number;
  /** Extensions played by the webview itself, so the backend only has to hand over the bytes. */
  audioExtensions: Array<string>;
  maximumAudioSize: number;
};

/**
 * Stored bytes that several file entries of one volume set locate at once.
 *
 * Derived from the descriptors, never recorded by a writer: the format has no alias field, so a packer that stored a
 * file once and pointed a second row at it left only equal fields behind. Calling this "aliased" would claim to know
 * what the packer did; it knows only what a reader does, which is read the same bytes for every name here.
 */
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

/** One archived text file read for display: its name, decoded content, and unpacked size. */
export type ProjectReadResult = {
  /** Entry name the content was read under. */
  name: string;
  /** Entry text decoded from Windows-1251, like every engine text format. */
  content: string;
  /** Entry bytes once unpacked, before decoding. */
  size: number;
};
