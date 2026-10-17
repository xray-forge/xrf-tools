import { getPathName } from "#/separator";

/**
 * Reads the extension out of a file name, in either separator style, by X-Ray's rule rather than the Unix one: a
 * leading dot names an extension rather than a hidden file, as in the engine's `shaders\r1\.s`.
 *
 * @param name - Engine entry name or host file name, `\` or `/` separated.
 * @returns The extension without its dot, or an empty string when the name has none.
 */
export function getFileExtension(name: string): string {
  // Reduced to the last segment first, so a dot in a directory name is not read as the extension.
  const segment: string = getPathName(name);
  const dot: number = segment.lastIndexOf(".");

  return dot < 0 ? "" : segment.slice(dot + 1);
}

/**
 * The same extension, folded the way the engine compares one.
 *
 * @param name - Engine entry name or host file name, `\` or `/` separated.
 * @returns The lower-cased extension, or an empty string when the name has none.
 */
export function getFoldedFileExtension(name: string): string {
  return getFileExtension(name).toLowerCase();
}
