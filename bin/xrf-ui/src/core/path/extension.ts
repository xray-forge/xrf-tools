import { Nullable } from "@xrf/types";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { getFoldedFileExtension } from "@/lib/path/extension";

/** Every declared spelling, keyed by itself, which is what a folded token is looked up in. */
const XRAY_EXTENSION_BY_SPELLING: ReadonlyMap<string, EXrayExtension> = new Map(
  Object.values(EXrayExtension).map((extension: EXrayExtension) => [extension as string, extension])
);

/**
 * The vocabulary member `name` carries, or null for a spelling nothing here models.
 *
 * @param name - Engine entry name or host file name, `\` or `/` separated.
 * @returns The declared extension the name carries, or null.
 */
export function getXrayExtension(name: string): Nullable<EXrayExtension> {
  return XRAY_EXTENSION_BY_SPELLING.get(getFoldedFileExtension(name)) ?? null;
}

/**
 * A path that names a format the backend reads, appending `fallback` to one that names none.
 *
 * @param path - The path as a save dialog returned it.
 * @param supported - The extensions the backend takes a format from.
 * @param fallback - The extension a path naming none of them takes.
 * @returns The path unchanged when its extension is supported, otherwise the path with the fallback appended.
 */
export function withSupportedExtension(
  path: string,
  supported: ReadonlyArray<EXrayExtension>,
  fallback: EXrayExtension
): string {
  const extension: Nullable<EXrayExtension> = getXrayExtension(path);

  return extension !== null && supported.includes(extension) ? path : `${path}.${fallback}`;
}
