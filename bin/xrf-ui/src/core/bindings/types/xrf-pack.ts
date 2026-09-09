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
  /** Everything the run took, which the three phase durations below divide between them. */
  duration: number;
  /** The share of `duration` spent walking the source tree, before anything was created. */
  collectDuration: number;
  /** The share of `duration` spent reading, compressing, and placing the selected entries. */
  writeDuration: number;
  /** The share of `duration` spent closing the last volume and naming the set. */
  finalizeDuration: number;
  /**
   * Source bytes per second over the whole run, so a reader compares two runs without dividing.
   *
   * Zero where the run took no measurable time, rather than a division a caller has to guard.
   */
  speed: number;
};

/** One entry a comparison classified, named by the identity both sides fold it to. */
export type ArchivePatchChange = {
  /** Engine identity: lower-case, backslash-separated, as `CLocatorAPI::Register` folds both spellings to. */
  name: string;
  class: ArchivePatchClass;
  base: ArchivePatchSide | null;
  target: ArchivePatchSide | null;
};

/** What a comparison decided about one engine identity. */
export type ArchivePatchClass =
  /** Only the target has it, so the patch carries it. */
  | "added"
  /** Both have it and their payloads differ, so the patch carries the target's. */
  | "modified"
  /** Only the base has it. The patch cannot carry this, and says so. */
  | "removed";

/** Comparison roots, entry filters, and patch volume settings. */
export type ArchivePatchConfig = {
  /** Root of the release to patch. Installations use `fsgame.ltx` mount order; later declarations win. */
  base: string;
  /** Root the new build is mounted from. */
  target: string;
  destination: string;
  /** Base name of the volumes, which become `<name>.db0`, `<name>.db1` and so on. */
  name: string;
  /** Logical prefixes the comparison is restricted to, or the whole of both worlds when empty. */
  include: Array<string>;
  /** Logical prefixes dropped from the comparison, applied after [`Self::include`]. */
  ignore: Array<string>;
  /** Extension patterns that keep a file out of the comparison, such as `*.txt`, matched with the dot. */
  excludeExtensions: Array<string>;
  /** Verbatim `[header]` text written as chunk 666, defaulting to the mountable gamedata header. */
  header: string | null;
  mode: ArchivePackMode;
  maxVolumeSize: number;
  isWithOversizedVolumes: boolean;
  volumeExtension: ArchiveVolumeExtension;
};

/**
 * One place a comparison read entries from, listed once per report and referred to by index.
 *
 * Mirrors [`XrayAssetContainer`] without its `relative_path`. That field is the reason a container cannot simply be
 * shared — it differs per entry — and it is also the reason sharing is worth arranging: nothing downstream reads it,
 * because the logical name is the identity every consumer of a comparison already works in.
 *
 * The saving is not marginal. A comparison of two real gamedata trees reported 34,513 changed entries across 46,352
 * sides, and named one of **two** distinct roots on every one of them: 6.3 MB of a 16.9 MB report, 37% of the file,
 * to say something a two-line table says once.
 */
export type ArchivePatchOrigin =
  /** A loose tree, named by the root it mounted at. */
  | { kind: "directory"; root: string }
  /** The archive volume set at `path`. */
  | { kind: "archive"; path: string };

/** Patch publication outcome, serialized with a `kind` tag. */
export type ArchivePatchPublication =
  /** No write was attempted: a comparison or a run cancelled before publication. */
  | { kind: "compared" }
  /** No added or modified entries required publication. Removed entries may still exist. */
  | { kind: "unnecessary" }
  /** Publication was attempted; the result records completion, cancellation, and retained volumes. */
  | ({
      kind: "published";
    } & ArchivePackResult);

/**
 * Archive comparison details and publication outcome. Empty change lists are serialized; unchanged entries are
 * counted.
 */
export type ArchivePatchResult = {
  /** Whether the run reached the end of its work or was stopped. */
  outcome: JobOutcome;
  /** Entries only the target holds, which the patch carries. */
  added: Array<ArchivePatchChange>;
  /** Entries both hold with differing payloads, which the patch carries from the target. */
  modified: Array<ArchivePatchChange>;
  /** Entries only the base holds. Reported but never deleted: the `.db` format cannot encode deletions. */
  removed: Array<ArchivePatchChange>;
  /** Entries both sides read identically, counted rather than listed. */
  unchanged: number;
  /**
   * Every volume set and loose root the run read from, which each side of each change names by index.
   *
   * Shared rather than repeated per entry: a comparison meets a handful of origins and classifies tens of thousands
   * of entries, so naming one on every side is most of a large report's weight.
   */
  origins: Array<ArchivePatchOrigin>;
  /** Entry pairs requiring a computed checksum. Excludes optional byte-for-byte verification reads. */
  payloadsRead: number;
  /**
   * Total unpacked size of added and modified target entries, including previews. Matches `size_source` for a
   * complete publication; archive size is determined when writing.
   */
  sizeCarried: number;
  /** What was done with the difference. */
  publication: ArchivePatchPublication;
  duration: number;
  /** The share of `duration` spent mounting both sides and deciding what differs. */
  compareDuration: number;
  /** The share of `duration` spent writing the difference into volumes, zero where none was written. */
  packDuration: number;
};

/** Which side of a comparison an entry was read from, and how big it was there. */
export type ArchivePatchSide = {
  /**
   * Position in the report's `origins` of the volume set or loose root this was read from.
   *
   * An index rather than the path itself: a comparison names a handful of origins over tens of thousands of entries,
   * so spelling one out per side is the bulk of a large report and says nothing a shared table cannot.
   */
  origin: number;
  /** Unpacked payload size, from the name table for an archived entry and from metadata for a loose one. */
  size: number;
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
