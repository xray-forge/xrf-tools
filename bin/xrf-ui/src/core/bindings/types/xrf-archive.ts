// Auto-generated rust bindings. Do not edit it manually.

/** One volume's parsed header: its entry table and the gamedata-relative root its `[header] entry_point` declares. */
export type ArchiveDescriptor = {
  /** Volume file creation time in Unix milliseconds, when the filesystem reports one. */
  createdAt: number | null;
  /** Volume file modification time in Unix milliseconds, when the filesystem reports one. */
  modifiedAt: number | null;
  /** Entries keyed by their authored name, exactly as the name table records them. */
  files: { [key in string]: ArchiveFileDescriptor };
  /** Root the volume unpacks under, from `[header] entry_point` with its alias stripped. */
  outputRootPath: string;
  /** The volume file this descriptor was read from. */
  path: string;
};

/**
 * One entry of a volume's name table: where its payload sits and how to verify it.
 *
 * Equal `size_real` and `size_compressed` is how the format says "stored uncompressed".
 */
export type ArchiveFileDescriptor = {
  /** CRC32 of the unpacked payload, recorded by the packer and verified on decompression. */
  crc: number;
  /** The volume file holding the payload. */
  source: string;
  /** Root the entry unpacks under, from its volume's header. */
  destination: string;
  /** Lower-cased extension derived from [`Self::name`], empty when the name has none. */
  extension: string;
  /**
   * Whether the entry names a directory rather than a file with bytes.
   *
   * A volume records the directories it contains so an unpacker can recreate them, and the engine writes those
   * entries with no payload — usually under the bare directory path, sometimes with a trailing separator.
   */
  isDirectory: boolean;
  /** Entry name as authored, which the engine registers verbatim. */
  name: string;
  /** Byte offset of the payload inside [`Self::source`]. */
  offset: number;
  /** Payload bytes as stored in the volume. */
  sizeCompressed: number;
  /** Payload bytes once unpacked. */
  sizeReal: number;
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
  archives: Array<ArchiveDescriptor>;
  files: { [key in string]: ArchiveFileDescriptor };
  readPolicy: ArchiveProjectReadPolicy;
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
 * both through the mounted asset world rather than through this project, and so answers to no limit of its own.
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

/** One archived text file read for display: its name, decoded content, and unpacked size. */
export type ProjectReadResult = {
  /** Entry name the content was read under. */
  name: string;
  /** Entry text decoded from Windows-1251, like every engine text format. */
  content: string;
  /** Entry bytes once unpacked, before decoding. */
  size: number;
};
