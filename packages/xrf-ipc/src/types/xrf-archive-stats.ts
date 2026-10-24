// Auto-generated rust bindings. Do not edit it manually.

/** What a volume set's payloads compress to. */
export type ArchiveCompression = {
  /** Bytes the payloads occupy as stored. */
  sizeCompressed: number;
  /** Bytes they occupy once unpacked. */
  sizeReal: number;
  /** Entries whose stored size equals their unpacked size, which is how the format says "stored uncompressed". */
  storedUncompressed: number;
};

/** How much of a subject one file extension accounts for. */
export type ArchiveExtensionUsage = {
  /** The spelling as found, lower-cased, or `None` for a name carrying no extension at all. */
  extension: string | null;
  /** Whether [`xrf_extension::XrayExtension`] declares this spelling. */
  isDeclared: boolean;
  measure: ArchiveMeasure;
  /** Stored bytes these entries occupy, for a subject that records them. */
  sizeCompressed: number | null;
};

/** How much of a subject one top-level folder accounts for. */
export type ArchiveFolderUsage = {
  /** The first path segment, or `None` for files sitting at the root of the tree. */
  folder: string | null;
  measure: ArchiveMeasure;
  /** Stored bytes, for a subject that records them. `None` for a mounted world. */
  sizeCompressed: number | null;
};

/** One of the largest files a subject holds. */
export type ArchiveLargestEntry = {
  /** Engine identity, as every other surface of the explorer addresses this file by. */
  name: string;
  /** Unpacked bytes. */
  sizeReal: number;
  /** Stored bytes, for a subject that records them. `None` for a mounted world. */
  sizeCompressed: number | null;
};

/** How many entries something holds, and what they weigh. */
export type ArchiveMeasure = {
  /** Entries counted. Directory entries are never among them; only files hold bytes. */
  files: number;
  /** Unpacked bytes those entries hold. */
  sizeReal: number;
};

/** Where a subject's files come from, and what its own search order hides. */
export type ArchiveOrigins = {
  /** Winning entries served from a loose file on disk. */
  loose: ArchiveMeasure;
  /** Winning entries served from inside a volume. */
  archived: ArchiveMeasure;
  /** Entries no lookup reaches, across every source. */
  hidden: ArchiveMeasure;
  /** One entry per source, in search priority order, so the row above is the one that wins. */
  sources: Array<ArchiveSourceUsage>;
};

/** What a subject holds, before any breakdown of it. */
export type ArchiveOverview = {
  /** Files and the bytes they hold, which is the total every breakdown sums back to. */
  total: ArchiveMeasure;
  /** Sources answered from: volumes for a volume set, mounts for a world. */
  sources: number;
  /** Entries naming a directory rather than a file. */
  directories: number;
  /** Files of zero length. */
  emptyFiles: number;
  /** Unpacked bytes of the largest single file. */
  largestFile: number;
  /** Mean unpacked bytes per file, zero when there are none. */
  meanFile: number;
  /** Median unpacked bytes per file, zero when there are none. */
  medianFile: number;
};

/** How much of a subject falls in one band of the file-size distribution. */
export type ArchiveSizeBand = {
  /** Smallest size this band admits, inclusive. */
  from: number;
  /** Smallest size the next band admits, or `None` for the open-ended top band. */
  to: number | null;
  measure: ArchiveMeasure;
};

/** How much of a subject one source accounts for, and how much of itself it loses to the sources above it. */
export type ArchiveSourceUsage = {
  /** The volume file, or the loose root, a copy sits in — the grain a container names, not the mount's. */
  source: string;
  /** Whether this source is a loose tree rather than a volume set. */
  isLoose: boolean;
  /** Entries of this source a lookup reaches. */
  wins: ArchiveMeasure;
  /** Entries of this source no lookup reaches, because a higher-priority source claims their engine path. */
  hides: ArchiveMeasure;
};

/** Everything a breakdown of one open subject reports. */
export type ArchiveStatistics = {
  overview: ArchiveOverview;
  /** One entry per extension found, heaviest first. */
  extensions: Array<ArchiveExtensionUsage>;
  /** One entry per top-level folder, heaviest first. */
  folders: Array<ArchiveFolderUsage>;
  /** The size distribution, in ascending band order, with empty bands omitted. */
  sizes: Array<ArchiveSizeBand>;
  /** The largest files, largest first. */
  largest: Array<ArchiveLargestEntry>;
  /** What the payloads compress to, for a volume set. `None` for a world, which records no stored size. */
  compression: ArchiveCompression | null;
  /** One entry per volume, in merge order. `None` for a world, whose sources are mounts rather than volumes. */
  volumes: Array<ArchiveVolumeSummary> | null;
  /**
   * Where the files come from and what the search order hides. Answered by both subjects: a volume set is an ordered
   * stack of volumes exactly as a world is an ordered stack of mounts.
   */
  origins: ArchiveOrigins;
};

/** One volume of a set, as its own name table recorded it. */
export type ArchiveVolumeSummary = {
  /** The volume file this summary describes. */
  path: string;
  /** Entries this volume's name table holds, directories included. */
  entries: number;
  /** Bytes its entries occupy as stored. */
  sizeCompressed: number;
  /** Bytes they occupy once unpacked. */
  sizeReal: number;
  /** Volume file modification time in Unix milliseconds, when the filesystem reports one. */
  modifiedAt: number | null;
};
