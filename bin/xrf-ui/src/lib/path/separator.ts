/**
 * The separator every X-Ray logical path and engine reference is written with.
 *
 * The engine's own convention rather than the host's: a reference like `ston\ston_beton05` is spelled this way on
 * every platform, because it is a name the game data carries rather than a path the filesystem resolved.
 */
export const LOGICAL_PATH_SEPARATOR: string = "\\";

/**
 * Both separators a path from the host may be written with.
 *
 * Windows accepts either and hands back either, depending on who built the string - a native dialog, a configuration
 * file, or a join done here - so anything reading a filesystem path has to accept both. This is the distinction worth
 * keeping from {@link LOGICAL_PATH_SEPARATOR}: an engine reference is never `/`-separated, and a host path may be.
 */
export const PATH_SEPARATORS: ReadonlyArray<string> = ["\\", "/"];

/** Matches every position just after a separator of either kind, which is where a path may be broken. */
const AFTER_SEPARATOR: RegExp = /(?<=[\\/])/;

/**
 * Where a host path's last separator is, whichever kind it is written with.
 *
 * The primitive under "the file's own name", "the directory above it" and "the path without its extension", each of
 * which was spelling this twice per call site. Those three stay separate operations, because they mean different
 * things; what they share is only knowing where a path stops being a directory.
 *
 * @param path - A path from the host, in either separator style.
 * @returns The index of the last separator, or -1 when the path carries none.
 */
export function findLastSeparator(path: string): number {
  return Math.max(...PATH_SEPARATORS.map((separator: string) => path.lastIndexOf(separator)));
}

/**
 * The last segment of a path: the file's own name, whichever separator style placed it.
 *
 * @param path - A path from the host, in either separator style.
 * @returns The segment after the last separator, or the whole path when it carries none.
 */
export function getPathName(path: string): string {
  return path.slice(findLastSeparator(path) + 1);
}

/**
 * A path cut after each separator, so a surface can offer those points as places to break it.
 *
 * Keeps every character, separators included: the segments joined back together are the path that went in. It exists
 * because a long path rendered in a narrow column otherwise breaks mid-name, which reads as a different name.
 *
 * @param path - A path or engine reference, in either separator style.
 * @returns Its segments, each ending in the separator that closed it, or one segment when it has none.
 */
export function splitAfterSeparators(path: string): Array<string> {
  return path.split(AFTER_SEPARATOR);
}
