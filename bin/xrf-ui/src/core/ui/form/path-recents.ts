import { parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/lib/local-storage";
import { isSamePath } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

/**
 * How many paths one field keeps.
 *
 * The same number the menu shows: nothing filters the list, so a record past the end of the menu could never be
 * picked and would only occupy storage.
 */
export const RECENT_PATHS_LIMIT: number = 10;

/** One path a field was given, and when. */
export interface IPathRecord {
  path: string;
  /** Milliseconds since the epoch, for saying how old the entry is. Order is carried by position, not by this. */
  at: number;
}

/**
 * A field's history as the surfaces that offer it see it.
 */
export interface IPathFieldRecents {
  /** The remembered paths, newest first. */
  records: ReadonlyArray<IPathRecord>;
  /** Takes one of them. */
  pick: (path: string) => void;
  /** Drops one of them. */
  forget: (path: string) => void;
}

/**
 * Reads one field's history, tolerating anything.
 *
 * A history is a convenience, so unusable content is absent content: a list that does not parse reads as empty, and a
 * record that is not one is dropped on its own rather than taking the rest with it. Duplicates left by an older
 * writer, or by a hand edit, collapse here too, so a caller never has to.
 *
 * @param key - Storage key to read.
 * @returns The stored records, newest first, at most {@link RECENT_PATHS_LIMIT} of them.
 */
export function readRecentPaths(key: string): Array<IPathRecord> {
  const stored: Nullable<unknown> = parseLocalStorageValueSafe<unknown>(key);

  if (!Array.isArray(stored)) {
    return [];
  }

  return stored.reduce((records: Array<IPathRecord>, candidate: unknown) => {
    const record: Nullable<IPathRecord> = toRecord(candidate);

    if (record && records.length < RECENT_PATHS_LIMIT && !contains(records, record.path)) {
      records.push(record);
    }

    return records;
  }, []);
}

/**
 * Stores one field's history, reporting a failure rather than raising it.
 *
 * @param key - Storage key to write.
 * @param records - Records to persist, newest first.
 */
export function writeRecentPaths(key: string, records: ReadonlyArray<IPathRecord>): void {
  setLocalStorageValueSafe(key, records.length ? JSON.stringify(records) : null);
}

/**
 * The history with one path at its head.
 *
 * A path already listed moves rather than repeating, because the two spellings of one directory are one entry and the
 * newer visit is the one worth dating. The oldest record leaves once the list is full.
 *
 * @param records - The history as it stands, newest first.
 * @param path - The path that was just used.
 * @param at - When it was used.
 * @returns The new history, newest first.
 */
export function recordRecentPath(records: ReadonlyArray<IPathRecord>, path: string, at: number): Array<IPathRecord> {
  if (!path) {
    return [...records];
  }

  return [{ at, path }, ...forgetRecentPath(records, path)].slice(0, RECENT_PATHS_LIMIT);
}

/**
 * The history without one path.
 *
 * @param records - The history as it stands.
 * @param path - The path to forget.
 * @returns The new history.
 */
export function forgetRecentPath(records: ReadonlyArray<IPathRecord>, path: string): Array<IPathRecord> {
  return records.filter((it: IPathRecord) => !isSamePath(it.path, path));
}

/** Whether a path is already listed, by the identity the history compares with. */
function contains(records: ReadonlyArray<IPathRecord>, path: string): boolean {
  return records.some((it: IPathRecord) => isSamePath(it.path, path));
}

/** Reads a stored element as a record, or answers `null` when it is not one. */
function toRecord(candidate: unknown): Nullable<IPathRecord> {
  if (typeof candidate !== "object" || candidate === null) {
    return null;
  }

  const { path, at } = candidate as Partial<IPathRecord>;

  if (typeof path !== "string" || !path || typeof at !== "number" || !Number.isFinite(at)) {
    return null;
  }

  return { at, path };
}
