import { getFileExtension } from "@/lib/path/extension";

/** Formats the extern manifest can be written as, in the order the save dialog offers them. */
export const EXTERN_MANIFEST_FORMATS: ReadonlyArray<string> = ["json", "xml", "html"];

/** The format a destination takes when the save dialog returned a name with nothing the backend can read. */
export const DEFAULT_EXTERN_MANIFEST_FORMAT: string = "json";

/** Name the artifact is checked in under, which is what a project already has beside its declarations. */
export const DEFAULT_MANIFEST_NAME: string = `extern.${DEFAULT_EXTERN_MANIFEST_FORMAT}`;

/** Filters the save dialog offers, one entry per format, so the chosen extension is what picks the writer. */
export const MANIFEST_FILTERS = EXTERN_MANIFEST_FORMATS.map((format: string) => ({
  name: `${format.toUpperCase()} manifest`,
  extensions: [format],
}));

/**
 * Extensions the backend infers a format from.
 *
 * `htm` is accepted but not offered: a person can type it, and turning it into `.htm.json` would write HTML's twin
 * under a name saying otherwise.
 */
const SUPPORTED_EXTENSIONS: ReadonlyArray<string> = [...EXTERN_MANIFEST_FORMATS, "htm"];

/**
 * Appends the default format when the destination names none the backend can read.
 *
 * @param path - Destination as the save dialog returned it.
 * @returns The original path when its extension names a format, otherwise the path with `.json` appended.
 */
export function withExternManifestExtension(path: string): string {
  return SUPPORTED_EXTENSIONS.includes(getFileExtension(path)) ? path : `${path}.${DEFAULT_EXTERN_MANIFEST_FORMAT}`;
}
