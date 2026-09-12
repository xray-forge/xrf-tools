import { ARCHIVE_PACK_MODE, ARCHIVE_VOLUME_EXTENSION, DEFAULT_ENTRY_POINT } from "@/core/archive/lib";
import { ArchivePackConfig, ArchivePackDirectory } from "@/core/bindings/types/xrf-pack";
import { BYTES_PER_MEGABYTE } from "@/lib/memory/size";

/**
 * Supported pack configuration extensions.
 */
export const PACK_CONFIG_EXTENSIONS: ReadonlyArray<string> = ["ltx", "json"];

/**
 * The extension a configuration takes when the save dialog returned a bare name.
 */
export const DEFAULT_PACK_CONFIG_EXTENSION: string = "ltx";

/**
 * What the editor opens on before the packer answers with its own defaults.
 */
export const FALLBACK_PACK_CONFIG: ArchivePackConfig = {
  source: "",
  destination: "",
  name: "gamedata",
  includeFiles: [],
  includeDirectories: [],
  excludeDirectories: [],
  excludeExtensions: [],
  isWithSkipList: true,
  header: `[header]\r\nauto_load = true\r\nentry_point = ${DEFAULT_ENTRY_POINT}\r\n`,
  mode: ARCHIVE_PACK_MODE.Compress,
  maxVolumeSize: 1900 * BYTES_PER_MEGABYTE,
  volumeExtension: ARCHIVE_VOLUME_EXTENSION.Db,
};

/**
 * Replaces one directory rule in place.
 *
 * @param directories - Rules the configuration holds.
 * @param index - Position of the rule to change.
 * @param patch - Fields to write over that rule.
 * @returns A new list with the rule at `index` updated.
 */
export function withDirectoryAt(
  directories: Array<ArchivePackDirectory>,
  index: number,
  patch: Partial<ArchivePackDirectory>
): Array<ArchivePackDirectory> {
  return directories.map((directory, at) => (at === index ? { ...directory, ...patch } : directory));
}

/**
 * Appends the default extension when the path lacks a supported suffix.
 *
 * @param path - Export path.
 * @returns The original path if supported, otherwise the path with `.ltx` appended.
 */
export function withPackConfigExtension(path: string): string {
  const extension: string = path.split(/[\\/]/).pop()?.split(".").slice(1).pop()?.toLowerCase() ?? "";

  return PACK_CONFIG_EXTENSIONS.includes(extension) ? path : `${path}.${DEFAULT_PACK_CONFIG_EXTENSION}`;
}

/**
 * Checks whether selection covers the source root; exclusion rules still apply.
 *
 * @param config - Pack configuration.
 * @returns Whether both include lists are empty.
 */
export function isWholeDirectory(config: ArchivePackConfig): boolean {
  return !config.includeDirectories.length && !config.includeFiles.length;
}
