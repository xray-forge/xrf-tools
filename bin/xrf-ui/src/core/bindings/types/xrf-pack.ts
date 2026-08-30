// Auto-generated rust bindings. Do not edit it manually.

/** What extracting one archived directory produced. */
export type ArchiveExtractDirectoryResult = {
  prefix: string;
  destination: string;
  extractedCount: number;
  size: number;
};

/** What extracting one archived file produced. */
export type ArchiveExtractResult = {
  name: string;
  destination: string;
  size: number;
};

/**
 * Everything needed to pack one archive volume set.
 *
 * Built from defaults, then optionally from an xrCompress LTX, then from explicit parameters, so a
 * command line and a form can layer over the same config file in the same order.
 *
 * Also the wire contract the desktop editor holds: it is read from a configuration file, edited in
 * place, packed, and written back, so all three surfaces speak one shape.
 */
export type ArchivePackConfig = {
  /** Root the archived names are relative to, normally a `gamedata` directory. */
  source: string;
  destination: string;
  /**
   * Base name of the volumes, which become `<name>.db0`, `<name>.db1` and so on.
   *
   * One host file name, never a path: it is joined to `destination`, and packing refuses anything that would
   * resolve elsewhere.
   */
  name: string;
  includeFiles: Array<string>;
  includeDirectories: Array<ArchivePackDirectory>;
  excludeDirectories: Array<ArchivePackDirectory>;
  /** Extension patterns from `[options] exclude_exts`, matched against the extension with its dot. */
  excludeExtensions: Array<string>;
  /** Apply the skip rules xrCompress hard-codes for editor and source leftovers. */
  isWithSkipList: boolean;
  /** Verbatim `[header]` text written as chunk 666. */
  header: string | null;
  mode: ArchivePackMode;
  /**
   * Hard maximum for a produced volume file, counting every byte it holds: chunk headers, header text, payloads as
   * they are actually stored, and the descriptor table written last. Packing refuses a cap it cannot keep rather
   * than exceeding it, which is stricter than the target xrCompress tests before each file and routinely overshoots.
   */
  maxVolumeSize: number;
  volumeExtension: ArchiveVolumeExtension;
};

/**
 * One `[include_folders]` or `[exclude_folders]` entry.
 *
 * The section names keep the engine's spelling because they are the xrCompress dialect; everything this crate names
 * itself says `directory`.
 *
 * The boolean has a different meaning on each side, which is an xrCompress quirk worth stating: an
 * included directory recurses into subdirectories, while an excluded one covers everything below itself rather than
 * only the directory it names. Either way the path is matched on complete components, without case.
 */
export type ArchivePackDirectory = {
  path: string;
  isRecursive: boolean;
};

/** How file payloads are stored in the archive. */
export type ArchivePackMode =
  /** Compress what the engine expects to be compressed and store the rest. */
  | "Compress"
  /** Store everything, the `-store` flag of xrCompress. */
  | "Store";

/** What one packing run produced. */
export type ArchivePackResult = {
  /** Volumes written, in mount order. */
  volumes: Array<string>;
  filesTotal: number;
  /** Files the include, exclude, and skip rules left out. */
  filesSkipped: number;
  filesStored: number;
  filesCompressed: number;
  /** Files that shared an identical earlier payload and cost only a descriptor row. */
  filesAliased: number;
  sizeSource: number;
  sizeWritten: number;
  duration: number;
};

/**
 * What unpacking a whole archive project produced.
 *
 * The two path fields are rendered for a person through `xrf_utils::format_path`, never addresses: a
 * host name that is not valid Unicode renders lossily rather than failing a run whose files are already
 * on disk. A caller that needs to open the destination uses the path it supplied.
 */
export type ArchiveUnpackResult = {
  /** Volume files that were read, rendered for display. */
  archives: Array<string>;
  duration: number;
  /** Root the files were written under, rendered for display. */
  destination: string;
  prepareDuration: number;
  unpackedSize: number;
  unpackDuration: number;
};

/** Extension the produced volumes carry, which also decides how the engine treats a missing header. */
export type ArchiveVolumeExtension = "Db" | "Xdb";
