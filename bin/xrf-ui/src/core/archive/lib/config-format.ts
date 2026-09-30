import { DialogFilter } from "@tauri-apps/plugin-dialog";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { withSupportedExtension } from "@/core/path/extension";
import { toFormatFilters } from "@/core/path/file-filters";

/**
 * Formats an archive configuration can be written as, in the order a save dialog offers them.
 */
export const ARCHIVE_CONFIG_EXTENSIONS: ReadonlyArray<EXrayExtension> = [EXrayExtension.LTX, EXrayExtension.JSON];

/** Filters the save dialog offers, one entry per format. */
export const ARCHIVE_CONFIG_EXPORT_FILTERS: Array<DialogFilter> = toFormatFilters(
  ARCHIVE_CONFIG_EXTENSIONS,
  (format: EXrayExtension) => format
);

/** The extension a configuration takes when the save dialog returned a bare name. */
export const DEFAULT_ARCHIVE_CONFIG_EXTENSION: EXrayExtension = EXrayExtension.LTX;

/**
 * Appends the default extension when the path names no format the backend can write.
 *
 * @param path - Export path as the save dialog returned it.
 * @returns The original path when it already names a format, otherwise the path with the default one appended.
 */
export function withArchiveConfigExtension(path: string): string {
  return withSupportedExtension(path, ARCHIVE_CONFIG_EXTENSIONS, DEFAULT_ARCHIVE_CONFIG_EXTENSION);
}
