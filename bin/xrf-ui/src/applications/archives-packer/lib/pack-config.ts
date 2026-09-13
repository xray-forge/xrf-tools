import { DEFAULT_ENTRY_POINT } from "@/core/archive/lib";
import {
  ArchivePackConfig,
  ArchivePackDirectory,
  EArchivePackMode,
  EArchiveVolumeExtension,
} from "@/core/ipc/types/xrf-pack";
import { BYTES_PER_MEGABYTE } from "@/lib/memory/size";

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
  mode: EArchivePackMode.COMPRESS,
  maxVolumeSize: 1900 * BYTES_PER_MEGABYTE,
  volumeExtension: EArchiveVolumeExtension.DB,
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
 * Checks whether selection covers the source root; exclusion rules still apply.
 *
 * @param config - Pack configuration.
 * @returns Whether both include lists are empty.
 */
export function isWholeDirectory(config: ArchivePackConfig): boolean {
  return !config.includeDirectories.length && !config.includeFiles.length;
}
