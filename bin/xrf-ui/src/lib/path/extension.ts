/**
 * Reads the extension out of a file name, in either separator style.
 *
 * The backend used to carry this per entry, which cost one string allocation for every name in an opened archive and
 * was read by nothing on the Rust side. It is a pure function of the name, so the name is all that crosses the wire
 * now and the derivation lives here.
 *
 * @param name - Archive entry name or host file name, `\` or `/` separated.
 * @returns The extension without its dot and lower-cased, or an empty string when the name has none.
 */
export function getFileExtension(name: string): string {
  // Reduced to the last segment first, so a dot in a directory name is not read as the extension.
  const segment: string = name.slice(Math.max(name.lastIndexOf("\\"), name.lastIndexOf("/")) + 1);
  const dot: number = segment.lastIndexOf(".");

  // A leading dot names a hidden file rather than an extension, so index zero is not a separator.
  return dot < 1 ? "" : segment.slice(dot + 1).toLowerCase();
}
