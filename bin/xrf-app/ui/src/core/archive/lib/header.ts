import { ArchivePackHeader, ArchivePackHeaderEntry } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";

/** Header key the engine reads to decide where an archive mounts. */
export const HEADER_ENTRY_POINT: string = "entry_point";

/** Header key the engine reads to decide whether to mount the archive at startup at all. */
export const HEADER_AUTO_LOAD: string = "auto_load";

/** Keys an editor gives a control of their own, so the rest can be listed as custom values. */
export const RESERVED_HEADER_KEYS: ReadonlyArray<string> = [HEADER_ENTRY_POINT, HEADER_AUTO_LOAD];

/** What a packed `gamedata` tree mounts as, and the value the engine expects for one. */
export const DEFAULT_ENTRY_POINT: string = "$fs_root$\\gamedata\\";

/**
 * Whether two header keys are the same key to an editor, with case ignored.
 *
 * The engine matches keys exactly, so this is a guard rather than its rule: a key differing from a present one only in
 * case is refused as a duplicate, and a control writing `entry_point` or `auto_load` replaces a folded match with the
 * engine's own spelling. `ArchivePackHeader::with_overrides` folds `--header` keys the same way, and the packer refuses
 * a header naming one key twice in any case.
 *
 * @param left - One key, as written.
 * @param right - The other.
 * @returns Whether they are the same key.
 */
export function isSameHeaderKey(left: string, right: string): boolean {
  return left.trim().toLowerCase() === right.trim().toLowerCase();
}

/**
 * @param key - A header key, as written.
 * @returns Whether an editor gives it a control of its own.
 */
export function isReservedHeaderKey(key: string): boolean {
  return RESERVED_HEADER_KEYS.some((reserved: string) => isSameHeaderKey(key, reserved));
}

/**
 * Reads the first matching header value.
 *
 * @param header - Header entries, or `null` for volumes without a header.
 * @param key - Key, matched with case ignored.
 * @returns The value, or `null` when the key is absent or blank: a blank entry point names nothing to the engine either.
 */
export function readHeaderValue(header: Nullable<ArchivePackHeader>, key: string): Nullable<string> {
  return header?.find((entry: ArchivePackHeaderEntry) => isSameHeaderKey(entry.key, key))?.value || null;
}

/**
 * Replaces a header key where it stands, or appends it when absent.
 *
 * Replaced in place so that editing one value keeps the order the header was written in, which an editor lists it in.
 * The value is kept exactly as typed, blank included, so a field being cleared to retype keeps its row; removal is
 * {@link removeHeaderEntry}. The packer judges what the engine would read from it.
 *
 * @param header - Current header entries, or `null` for none.
 * @param key - Key to replace, matched with case ignored and written as given.
 * @param value - Value to write.
 * @returns Updated header, with any later entry folding to the same key dropped.
 */
export function writeHeaderValue(header: Nullable<ArchivePackHeader>, key: string, value: string): ArchivePackHeader {
  const written: ArchivePackHeaderEntry = { key, value };
  const entries: ArchivePackHeader = [];
  let isReplaced: boolean = false;

  for (const entry of header ?? []) {
    if (!isSameHeaderKey(entry.key, key)) {
      entries.push(entry);
    } else if (!isReplaced) {
      // A repeated key collapses into the first, which is the one `readHeaderValue` answers with.
      isReplaced = true;
      entries.push(written);
    }
  }

  if (!isReplaced) {
    entries.push(written);
  }

  return entries;
}

/**
 * Removes every entry of a key.
 *
 * @param header - Current header entries, or `null` for none.
 * @param key - Key to remove, matched with case ignored.
 * @returns Updated header, or `null` when no entry remains, which writes volumes without a header.
 */
export function removeHeaderEntry(header: Nullable<ArchivePackHeader>, key: string): Nullable<ArchivePackHeader> {
  const entries: ArchivePackHeader = (header ?? []).filter(
    (entry: ArchivePackHeaderEntry) => !isSameHeaderKey(entry.key, key)
  );

  return entries.length ? entries : null;
}

/**
 * Reads a header flag.
 *
 * @param header - Header entries, or `null` for none.
 * @param key - Key, matched with case ignored.
 * @returns Whether the value is `true`, `on`, `yes`, or `1`, ignoring case.
 */
export function readHeaderFlag(header: Nullable<ArchivePackHeader>, key: string): boolean {
  const value: Nullable<string> = readHeaderValue(header, key);

  return value !== null && ["true", "on", "yes", "1"].includes(value.toLowerCase());
}

/**
 * Writes a header flag as explicit `true` or `false`.
 *
 * @param header - Current header entries, or `null` for none.
 * @param key - Key to replace.
 * @param isEnabled - Flag value.
 * @returns Updated header.
 */
export function writeHeaderFlag(
  header: Nullable<ArchivePackHeader>,
  key: string,
  isEnabled: boolean
): ArchivePackHeader {
  return writeHeaderValue(header, key, isEnabled ? "true" : "false");
}
