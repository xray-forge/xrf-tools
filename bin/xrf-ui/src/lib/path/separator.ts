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

/** The separator a comparison key is written with. */
const COMPARISON_SEPARATOR: string = "\\";

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

/**
 * The directory a path sits in, whichever separator style placed it.
 *
 * Keeps the separator that closes a root, because `C:` names the current directory of a drive rather than its root.
 * Answers an empty string for a path with no separator at all, which is a name rather than a location.
 *
 * @param path - A path from the host, in either separator style.
 * @returns The directory above it, or an empty string when the path names no directory.
 */
export function getPathDirectory(path: string): string {
  const separator: number = findLastSeparator(path);

  if (separator < 0) {
    return "";
  }

  const directory: string = path.slice(0, separator);

  return !directory || directory.endsWith(":") ? path.slice(0, separator + 1) : directory;
}

/**
 * One spelling of a host path, for deciding whether two paths name the same thing.
 *
 * Windows reaches the same directory through several spellings - either separator, a trailing one or not, any casing -
 * and a native dialog, a typed path and a path joined here disagree freely. This folds those apart, and is only ever a
 * key: what gets stored and shown stays the string that arrived.
 *
 * Wrong on Linux in two ways, both accepted: sibling paths differing only in case fold together, and a file name
 * containing `\` reads as two segments. Both cost a comparison, never a path, and the alternative is asking the backend
 * which platform this is before a key can be computed. Repeated separators are left alone, because collapsing them
 * would eat the `\\` a UNC path opens with.
 *
 * @param path - A path from the host, in either separator style.
 * @returns Its comparison key.
 */
export function toComparablePath(path: string): string {
  const unified: string = path.split("/").join(COMPARISON_SEPARATOR);

  let end: number = unified.length;

  while (end > 0 && unified[end - 1] === COMPARISON_SEPARATOR) {
    end -= 1;
  }

  const trimmed: string = unified.slice(0, end);

  // A root is all separator, or all drive letter and separator, so trimming would name something else entirely:
  // `C:` is the current directory of a drive, not its root.
  if (!trimmed || trimmed.endsWith(":")) {
    return (trimmed + COMPARISON_SEPARATOR).toLowerCase();
  }

  return trimmed.toLowerCase();
}

/**
 * Whether two host paths name the same thing.
 *
 * The comparison {@link toComparablePath} exists for, given a name so callers do not each remember to fold before
 * comparing - and inherit its two accepted Linux mistakes along with its Windows correctness.
 *
 * @param left - A path from the host.
 * @param right - Another path from the host.
 * @returns Whether they name one thing.
 */
export function isSamePath(left: string, right: string): boolean {
  return toComparablePath(left) === toComparablePath(right);
}

/**
 * A path shortened from the front, so that what identifies it survives.
 *
 * The opposite end from the usual ellipsis, and for the reason {@link getPathName} exists: the tail of a path is what
 * tells two of them apart, and a column of paths cut at the right hand end reads as the same path repeated. Whole
 * segments are kept where they fit, because a name cut in half reads as a different name.
 *
 * @param path - A path from the host, in either separator style.
 * @param limit - The most characters the result may occupy, ellipsis included.
 * @returns The path, or its tail behind an ellipsis.
 */
export function truncatePathHead(path: string, limit: number): string {
  if (path.length <= limit || limit < 2) {
    return path;
  }

  const segments: Array<string> = splitAfterSeparators(path);

  let tail: string = "";

  for (let index = segments.length - 1; index >= 0; index -= 1) {
    const candidate: string = segments[index] + tail;

    if (candidate.length + 1 > limit) {
      break;
    }

    tail = candidate;
  }

  // A single segment longer than the budget has no separator to break at, so it is cut mid-name after all.
  return `…${tail || path.slice(path.length - (limit - 1))}`;
}
