import { ArchivePackMode, ArchiveVolumeExtension } from "@/core/ipc/types/xrf-pack";

// todo: Improve tauri generator to expose enums instead of string types.

/**
 * Typed constants for the generated archive volume extensions.
 */
export const ARCHIVE_VOLUME_EXTENSION: { [K in ArchiveVolumeExtension]: K } = {
  Db: "Db",
  Xdb: "Xdb",
};

/** The extension a volume actually carries, mirroring `ArchiveVolumeExtension::as_str`. */
export const ARCHIVE_VOLUME_SUFFIX: { [K in ArchiveVolumeExtension]: string } = {
  Db: "db",
  Xdb: "xdb",
};

export const ARCHIVE_PACK_MODE: { [K in ArchivePackMode]: K } = {
  Compress: "Compress",
  Store: "Store",
};
