// Auto-generated rust bindings. Do not edit it manually.

/** A week unused, as Unreal's local derived data cache keeps eight days; five gibibytes, ccache's default. */
export const DEFAULT_APP_CACHE_POLICY: AppCachePolicy = { sizeLimit: 5368709120, unusedDays: 7 };

/** How long an application's caches keep what goes unused, and how much they keep at most. */
export type AppCachePolicy = {
  /** Days an entry may go unread before it is removed. */
  unusedDays: number;
  /** Bytes every cache together may hold, past which the least recently used entries are removed first. */
  sizeLimit: number;
};

/** What a tidy of an application's caches did: whether it ran, what it removed and what it kept. */
export type AppCacheTidy = {
  /** Whether it ran: not when another instance holds the caches, nor when one ran within the day and none was forced. */
  isDone: boolean;
  removedEntries: number;
  removedBytes: number;
  keptEntries: number;
  keptBytes: number;
};
