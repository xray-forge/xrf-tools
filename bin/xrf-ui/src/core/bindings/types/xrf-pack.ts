// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome } from "@/core/bindings/types/xrf-job";

/** What extracting one archived directory produced. */
export type ArchiveExtractDirectoryResult = {
  prefix: string;
  destination: string;
  /** Whether the run reached the end of what it selected or was stopped between entries. */
  outcome: JobOutcome;
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
  /**
   * Let `max_volume_size` exceed `VOLUME_SIZE_MAX`, for an engine fork that raised `XRP_MAX_SIZE`.
   *
   * Defaulted rather than required, because this shape is also a configuration file on disk: one written before the
   * field existed reads back as the safe answer instead of failing to parse.
   */
  isWithOversizedVolumes?: boolean;
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
  /**
   * Volumes written, in mount order.
   *
   * A volume appears here once it has been closed. On a forced run that stopped early this is the part of the set
   * that is structurally complete — not the part that is usable, since a set missing its later volumes is missing
   * entries. On any other run that did not finish it is empty, because such a run publishes nothing.
   */
  volumes: Array<string>;
  /**
   * Every volume path this run created, closed or not.
   *
   * Wider than `volumes` on purpose. A volume is opened with `File::create`, so it exists — and has replaced whatever
   * stood at that path — from the moment writing begins.
   *
   * Empty on a run that did not finish and was not forced: such a run began over a destination holding no volume of
   * its set, so every file it made was its own and was removed again. A forced run is where this earns its place —
   * there the same paths may have held a working set beforehand, deleting them would compound the loss, and the
   * caller needs the list to say what is now on disk.
   */
  volumesOpened: Array<string>;
  /**
   * Whether the run reached the end of its work or was stopped between entries.
   *
   * A cancelled pack publishes nothing and leaves the destination as it found it, unless it was forced — see
   * `volumes_opened` for what a forced run leaves behind.
   */
  outcome: JobOutcome;
  filesTotal: number;
  /** Files the include, exclude, and skip rules left out. */
  filesSkipped: number;
  filesStored: number;
  filesCompressed: number;
  /** Files that shared an identical earlier payload and cost only a descriptor row. */
  filesAliased: number;
  /** Bytes of every selected source file, the data the run had to read. */
  sizeSource: number;
  /** Bytes of every closed volume, headers and descriptor tables included. */
  sizeWritten: number;
  duration: number;
  /**
   * Source bytes per second over the whole run, so a reader compares two runs without dividing.
   *
   * Zero where the run took no measurable time, rather than a division a caller has to guard.
   */
  speed: number;
};

/**
 * What unpacking a whole archive project produced.
 *
 * The two path fields are rendered for a person through `xrf_utils::format_path`, never addresses: a
 * host name that is not valid Unicode renders lossily rather than failing a run whose files are already
 * on disk. A caller that needs to open the destination uses the path it supplied.
 *
 * Every count here describes what the run actually did, not what the project holds. That distinction only becomes
 * visible when a run stops early, which is exactly when a caller most needs the numbers to be true.
 */
export type ArchiveUnpackResult = {
  /** Volume files that were read, rendered for display. */
  archives: Array<string>;
  duration: number;
  /** Root the files were written under, rendered for display. */
  destination: string;
  /**
   * Whether the run reached the end of its work or was stopped at an entry boundary.
   *
   * A cancelled run leaves what it had already written where it is: the files below `destination` are a real but
   * partial tree, and nothing removes them. Read the counts below as what is on disk, never as a total.
   */
  outcome: JobOutcome;
  /** Entries dealt with, directory rows included, which is what the counts are measured against. */
  filesTotal: number;
  /** Files actually written. */
  filesUnpacked: number;
  prepareDuration: number;
  /** Bytes written, summed from the entries that were written rather than from the project. */
  unpackedSize: number;
  unpackDuration: number;
};

/** Extension the produced volumes carry, which also decides how the engine treats a missing header. */
export type ArchiveVolumeExtension = "Db" | "Xdb";
