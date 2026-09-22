import { Nullable } from "@xrf/types";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { getXrayExtension } from "@/core/path/extension";

/**
 * Formats an archive configuration can be written as, in the order a save dialog offers them.
 */
export const ARCHIVE_CONFIG_EXTENSIONS: ReadonlyArray<EXrayExtension> = [EXrayExtension.LTX, EXrayExtension.JSON];

/** The extension a configuration takes when the save dialog returned a bare name. */
export const DEFAULT_ARCHIVE_CONFIG_EXTENSION: EXrayExtension = EXrayExtension.LTX;

/**
 * Appends the default extension when the path names no format the backend can write.
 *
 * @param path - Export path as the save dialog returned it.
 * @returns The original path when it already names a format, otherwise the path with the default one appended.
 */
export function withArchiveConfigExtension(path: string): string {
  const extension: Nullable<EXrayExtension> = getXrayExtension(path);

  return extension !== null && ARCHIVE_CONFIG_EXTENSIONS.includes(extension)
    ? path
    : `${path}.${DEFAULT_ARCHIVE_CONFIG_EXTENSION}`;
}
