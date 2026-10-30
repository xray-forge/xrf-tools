import { systemCommands } from "@xrf/ipc/commands/system";
import { AppCachePolicy, AppCacheTidy } from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";
import { isRecord } from "@xrf/types";

import { APP_CACHE_POLICY_STORAGE_KEY, IStoredValueCodec, jsonCodec, StoredValue } from "@/core/storage";

/** A gibibyte, the unit the size limit is offered in. */
export const GIBIBYTE: number = 1024 ** 3;

/** The backend's own (`AppCachePolicy::DEFAULT`): a week unused, five gibibytes. */
export const DEFAULT_APP_CACHE_POLICY: Readonly<AppCachePolicy> = { sizeLimit: 5 * GIBIBYTE, unusedDays: 7 };

/** The bounds each limit is offered between: from a day to three months unused, from one gibibyte to fifty. */
export const APP_CACHE_POLICY_LIMITS = {
  sizeLimit: { max: 50 * GIBIBYTE, min: GIBIBYTE, step: GIBIBYTE },
  unusedDays: { max: 90, min: 1, step: 1 },
} as const;

/**
 * @param stored - What was stored, parsed.
 * @returns The limits as stored, each held to its bounds, the default's where one is missing or not a number.
 */
function toAppCachePolicy(stored: unknown): AppCachePolicy {
  const record: Record<string, unknown> = isRecord(stored) ? stored : {};

  function read(key: keyof AppCachePolicy): number {
    const value: unknown = record[key];
    const { max, min } = APP_CACHE_POLICY_LIMITS[key];

    return typeof value === "number" && Number.isFinite(value)
      ? Math.min(max, Math.max(min, Math.round(value)))
      : DEFAULT_APP_CACHE_POLICY[key];
  }

  return { sizeLimit: read("sizeLimit"), unusedDays: read("unusedDays") };
}

/** How the limits are kept: as JSON, read back held to their bounds. */
export const APP_CACHE_POLICY_CODEC: IStoredValueCodec<AppCachePolicy> = jsonCodec(toAppCachePolicy);

/**
 * Tidies the application's caches by the stored limits: what went unused past its age goes, then the least recently
 * used past the size limit. The backend runs it at most once a day unless `isForced`, and leaves it to another instance
 * holding the caches.
 *
 * @param isForced - Whether to tidy now rather than when a day has passed since the last.
 * @returns What the tidy did.
 */
export async function tidyAppCache(isForced: boolean): Promise<AppCacheTidy> {
  const policy: AppCachePolicy = new StoredValue(APP_CACHE_POLICY_STORAGE_KEY, APP_CACHE_POLICY_CODEC).value;

  return systemCommands.tidyAppCache(policy, isForced);
}

/** Tidies the caches as the page starts, when a day has passed since the last; a failure is logged, not shown. */
export function tidyAppCacheOnStart(): void {
  tidyAppCache(false).catch((error: unknown) => Logger.warn("Failed to tidy the application's caches:", error));
}
