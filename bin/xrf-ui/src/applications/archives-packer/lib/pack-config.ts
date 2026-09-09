import {
  ArchivePackConfig,
  ArchivePackDirectory,
  ArchivePackMode,
  ArchiveVolumeExtension,
} from "@/core/bindings/types/xrf-pack";
import { BYTES_PER_MEGABYTE } from "@/lib/memory/size";
import { Nullable } from "@/lib/types/general";

/**
 * Supported pack configuration extensions.
 */
export const PACK_CONFIG_EXTENSIONS: ReadonlyArray<string> = ["ltx", "json"];

/**
 * The extension a configuration takes when the save dialog returned a bare name.
 */
export const DEFAULT_PACK_CONFIG_EXTENSION: string = "ltx";

/**
 * Typed constants for the generated archive volume extensions.
 */
export const ARCHIVE_VOLUME_EXTENSION: { [K in ArchiveVolumeExtension]: K } = {
  Db: "Db",
  Xdb: "Xdb",
};

/** The extension a volume actually carries, mirroring `ArchiveVolumeExtension::as_str`. */
export const ARCHIVE_VOLUME_SUFFIX: { [K in ArchiveVolumeExtension]: string } = {
  Db: "db",
  Xdb: "xdb",
};

export const ARCHIVE_PACK_MODE: { [K in ArchivePackMode]: K } = {
  Compress: "Compress",
  Store: "Store",
};

/** Header key the engine reads to decide where an archive mounts. */
export const HEADER_ENTRY_POINT: string = "entry_point";

/** Header key the engine reads to decide whether to mount the archive at startup at all. */
export const HEADER_AUTO_LOAD: string = "auto_load";

/** Keys the editor gives a control of their own, so the rest can be listed as custom values. */
export const RESERVED_HEADER_KEYS: ReadonlyArray<string> = [HEADER_ENTRY_POINT, HEADER_AUTO_LOAD];

/** What a packed `gamedata` tree mounts as, and the value the engine expects for one. */
export const DEFAULT_ENTRY_POINT: string = "$fs_root$\\gamedata\\";

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
