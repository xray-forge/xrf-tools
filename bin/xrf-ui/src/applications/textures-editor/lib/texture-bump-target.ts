import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { Nullable } from "@/lib/types/general";

/** What the engine appends to a bump name to reach the other half of the pair. */
const COMPANION_SUFFIX: string = "#";

/** What the SDK's generator appends to a texture name to reach its pair. */
const BUMP_SUFFIX: string = "_bump";

/**
 * Where a generated pair is written: the texture's own path, without its extension.
 *
 * The backend appends `_bump` and `_bump#` to this, which is the SDK generator's own naming and the only naming the
 * two halves ever have. Null for a texture with no file on disk - one served out of an archive - because there is
 * nowhere beside it to write.
 *
 * @param description - The texture as the backend resolved it.
 * @returns The base path both halves are built from, or null when there is none.
 */
export function toBumpTarget(description: Nullable<TextureDescription>): Nullable<string> {
  const path: Nullable<string> = description?.targets?.texture.path ?? null;

  if (!path) {
    return null;
  }

  const dot: number = path.lastIndexOf(".");
  const separator: number = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));

  return dot > separator ? path.slice(0, dot) : path;
}

/**
 * The engine reference a generated pair answers to, for the descriptor to declare.
 *
 * The bump half only. The engine derives the companion itself by appending `#` (`uber_deffer.cpp`), and there is no
 * way to override that, so a descriptor names one and gets two.
 *
 * @param description - The texture the pair was generated for.
 * @returns The reference to write into the descriptor's bump field.
 */
export function toBumpReference(description: TextureDescription): string {
  return `${description.reference}${BUMP_SUFFIX}`;
}

/**
 * The companion reference the engine will derive from a bump name, for a surface that wants to name both.
 *
 * @param bumpReference - The bump half's reference.
 * @returns The companion's.
 */
export function toCompanionReference(bumpReference: string): string {
  return `${bumpReference}${COMPANION_SUFFIX}`;
}
