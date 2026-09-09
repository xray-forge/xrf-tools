/**
 * Formats a patching configuration can be written as, in the order the save dialog offers them.
 *
 * The same two the packer takes, so one habit covers both screens. `.ltx` here is not the xrCompress dialect — that
 * has no patching configuration to be compatible with — but the file it produces is read by the same parser, and the
 * sections say what they hold: `[include]`, `[ignore]`, `[options]` and a verbatim `[header]`.
 */
export const PATCH_CONFIG_EXTENSIONS: ReadonlyArray<string> = ["ltx", "json"];

/** The extension a configuration takes when the save dialog returned a bare name. */
export const DEFAULT_PATCH_CONFIG_EXTENSION: string = "ltx";

/**
 * Appends the default extension when the path lacks a supported suffix.
 *
 * @param path - Export path as the save dialog returned it.
 * @returns The original path when it already names a format, otherwise the path with `.ltx` appended.
 */
export function withPatchConfigExtension(path: string): string {
  const extension: string = path.split(/[\\/]/).pop()?.split(".").slice(1).pop()?.toLowerCase() ?? "";

  return PATCH_CONFIG_EXTENSIONS.includes(extension) ? path : `${path}.${DEFAULT_PATCH_CONFIG_EXTENSION}`;
}
