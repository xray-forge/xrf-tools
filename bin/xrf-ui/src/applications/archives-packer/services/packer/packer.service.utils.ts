import { ArchivePackConfig } from "@/core/ipc/types/xrf-pack";

/**
 * What a configuration file actually carries, as one comparable value.
 */
export function toSavedState(config: ArchivePackConfig): string {
  return JSON.stringify([
    config.includeDirectories,
    config.includeFiles,
    config.excludeDirectories,
    config.excludeExtensions,
    config.header,
  ]);
}
