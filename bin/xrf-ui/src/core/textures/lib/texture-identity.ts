import { TextureDescription } from "@/core/ipc/types/xrf-app";

/**
 * Identifies a texture by its address and ordered roots, independently of its display label.
 *
 * @param texture - The source and effective roots returned by the backend.
 * @returns A stable key for comparisons and editor drafts.
 */
export function getTextureIdentity(texture: Pick<TextureDescription, "source" | "roots">): string {
  const { source, roots } = texture;

  return JSON.stringify([
    source.kind,
    source.kind === "file" ? source.path : source.reference,
    roots.asset,
    roots.roots.map((root) => [root.path, root.mode ?? "auto"]),
  ]);
}
