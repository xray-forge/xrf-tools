import { Nullable } from "@xrf/types";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { findLastSeparator } from "@/lib/path/separator";

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
  const separator: number = findLastSeparator(path);

  return dot > separator ? path.slice(0, dot) : path;
}
