import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { getFoldedFileExtension } from "@/lib/path/extension";
import { Nullable } from "@/lib/types/general";

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
 * Whether `name` carries `extension`, compared the way `XrayExtension::matches` compares one.
 *
 * @param name - Engine entry name or host file name, `\` or `/` separated.
 * @param extension - The declared spelling to test for.
 * @returns Whether the name's extension is that one.
 */
export function hasXrayExtension(name: string, extension: EXrayExtension): boolean {
  return getFoldedFileExtension(name) === extension;
}
