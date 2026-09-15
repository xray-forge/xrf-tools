import { ETextureSource, TextureDescription, TextureSource } from "@/core/ipc/types/xrf-app";

/**
 * Identifies the texture a source names, independently of the roots it was resolved in.
 *
 * @param source - Where a texture is named from.
 * @returns A stable key for comparisons.
 */
export function getTextureSourceKey(source: TextureSource): string {
  return source.kind === ETextureSource.FILE ? `file:${source.path}` : `asset:${source.reference}`;
}

/**
 * Identifies a texture by its address and ordered roots, independently of its display label.
 *
 * @param texture - The source and effective roots returned by the backend.
 * @returns A stable key for comparisons and editor drafts.
 */
export function getTextureIdentity(texture: Pick<TextureDescription, "source" | "roots">): string {
  const { source, roots } = texture;

  return JSON.stringify([
    getTextureSourceKey(source),
    roots.asset,
    roots.roots.map((root) => [root.path, root.mode ?? "auto"]),
  ]);
}
