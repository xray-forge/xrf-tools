import { Nullable } from "@/lib/types/general";

/** Header key the engine reads to decide where an archive mounts. */
export const HEADER_ENTRY_POINT: string = "entry_point";

/** Header key the engine reads to decide whether to mount the archive at startup at all. */
export const HEADER_AUTO_LOAD: string = "auto_load";

/** Keys an editor gives a control of their own, so the rest can be listed as custom values. */
export const RESERVED_HEADER_KEYS: ReadonlyArray<string> = [HEADER_ENTRY_POINT, HEADER_AUTO_LOAD];

/** What a packed `gamedata` tree mounts as, and the value the engine expects for one. */
export const DEFAULT_ENTRY_POINT: string = "$fs_root$\\gamedata\\";

/**
 * Reads the first matching header value.
 *
 * @param header - Header text, or `null`.
 * @param key - Case-sensitive key.
 * @returns The trimmed value, or `null` when absent.
 */
export function readHeaderValue(header: Nullable<string>, key: string): Nullable<string> {
  if (!header) {
    return null;
  }

  for (const line of header.split(/\r?\n/)) {
    const [name, ...rest] = line.split("=");

    if (rest.length && name.trim() === key) {
      return rest.join("=").trim();
    }
  }

  return null;
}

/**
 * Replaces a header key, or removes it for a blank value. Rebuilds the section with CRLF line
 * endings.
 *
 * @param header - Current header text.
 * @param key - Key to replace.
 * @param value - Value to trim and write.
 * @returns Updated header, or `null` if no lines remain.
 */
export function writeHeaderValue(header: Nullable<string>, key: string, value: string): Nullable<string> {
  const lines: Array<string> = (header ?? "")
    .split(/\r?\n/)
    .filter((line) => line.trim() && !line.trim().startsWith("["))
    .filter((line) => line.split("=")[0]?.trim() !== key);

  if (value.trim()) {
    lines.push(`${key} = ${value.trim()}`);
  }

  if (!lines.length) {
    return null;
  }

  return `[header]\r\n${lines.join("\r\n")}\r\n`;
}

/**
 * Reads a header flag.
 *
 * @param header - Header text, or `null`.
 * @param key - Case-sensitive key.
 * @returns Whether the value is `true`, `on`, `yes`, or `1`, ignoring case.
 */
export function readHeaderFlag(header: Nullable<string>, key: string): boolean {
  const value: Nullable<string> = readHeaderValue(header, key);

  return value !== null && ["true", "on", "yes", "1"].includes(value.toLowerCase());
}

/**
 * Writes a header flag as explicit `true` or `false`.
 *
 * @param header - Current header text.
 * @param key - Key to replace.
 * @param isEnabled - Flag value.
 * @returns Updated header text.
 */
export function writeHeaderFlag(header: Nullable<string>, key: string, isEnabled: boolean): Nullable<string> {
  return writeHeaderValue(header, key, isEnabled ? "true" : "false");
}

/**
 * Reads header key-value pairs in line order.
 *
 * @param header - Header text, or `null`.
 * @returns Trimmed pairs from lines containing `=`.
 */
export function readHeaderEntries(header: Nullable<string>): Array<[string, string]> {
  if (!header) {
    return [];
  }

  return header
    .split(/\r?\n/)
    .map((line) => line.split("="))
    .filter((parts) => parts.length > 1)
    .map((parts) => [parts[0].trim(), parts.slice(1).join("=").trim()] as [string, string]);
}
