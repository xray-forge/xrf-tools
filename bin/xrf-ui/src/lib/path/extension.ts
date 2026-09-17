import { getPathName } from "@/lib/path/separator";

/**
 * Reads the extension out of a file name, in either separator style.
 * X-Ray's rule rather than the Unix one.
 *
 * The backend used to carry this per entry, which cost one string allocation for every name in an opened archive and
 * was read by nothing on the Rust side. It is a pure function of the name, so the name is all that crosses the wire
 * now and the derivation lives here.
 *
 * A leading dot names an extension rather than a hidden file: the engine ships `shaders\r1\.s`, which it loads like
 * any other shader, and nothing in game data is a Unix dotfile.
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
