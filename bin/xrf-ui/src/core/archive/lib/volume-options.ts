import { EArchiveVolumeExtension } from "@/core/ipc/types/xrf-pack";

/**
 * The extension a volume actually carries, mirroring `ArchiveVolumeExtension::as_str`.
 *
 * The vocabulary declares no member for either: a volume is `<name>.db0`, so what is stored here is an extension
 * stem the writer completes with an index rather than a spelling a file ever carries whole.
 */
export const ARCHIVE_VOLUME_SUFFIX: Readonly<Record<EArchiveVolumeExtension, string>> = {
  [EArchiveVolumeExtension.DB]: "db",
  [EArchiveVolumeExtension.XDB]: "xdb",
};

/**
 * How many indexed volumes a dialog offers per stem: every two-digit index.
 *
 * The writer numbers volumes without a limit (`ArchivePackConfig::volume_name`) and the reader takes any `db*` or
 * `xdb*` (`ArchiveDescriptor::is_valid_db_path`), but a dialog filter lists spellings, not patterns. A hundred
 * volumes at the default size is 190 GB, past any game tree; a set past it opens through `All files`.
 */
export const VOLUME_INDEX_COUNT: number = 100;

/**
 * Every file name suffix a published volume set can carry, for the dialog filters that browse one.
 *
 * The unindexed stem is offered too, because a set small enough to fit one volume is written without an index.
 */
export const ARCHIVE_VOLUME_FILE_EXTENSIONS: ReadonlyArray<string> = Object.values(ARCHIVE_VOLUME_SUFFIX).flatMap(
  (suffix: string) => [suffix, ...Array.from({ length: VOLUME_INDEX_COUNT }, (_, index: number) => `${suffix}${index}`)]
);
