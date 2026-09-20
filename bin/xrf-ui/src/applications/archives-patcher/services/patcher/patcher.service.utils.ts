import { ArchivePatchConfig } from "@/core/ipc/types/xrf-pack";

/**
 * What a configuration file actually carries, as one comparable value.
 */
export function toSavedState(config: ArchivePatchConfig): string {
  return JSON.stringify([config.include, config.ignore, config.excludeExtensions, config.header]);
}
