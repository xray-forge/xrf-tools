import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { VisualSubmesh, VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { getRenderSurface, IRenderSurface, isAlphaRenderSurface } from "@/core/render/lib/surface/render-surface";
import { Nullable } from "@/lib/types/general";

/**
 * The material state of every submesh, by the index the submesh reports.
 *
 * @param submeshes - Submeshes as the backend described them, in the order the model declares them.
 * @param surfaces - What the renderer draws for each submesh, answered in that same order.
 * @returns One material state per submesh.
 */
export function createVisualSurfaces(
  submeshes: Array<VisualSubmesh>,
  surfaces: ReadonlyArray<XraySurfaceDescriptor> = []
): Map<number, IRenderSurface> {
  return new Map(
    submeshes.map((submesh: VisualSubmesh, position: number) => [submesh.index, getRenderSurface(surfaces, position)])
  );
}

/**
 * Logical paths of the texture files a model reads alpha out of.
 *
 * @param surfaces - Material state per submesh index, as {@link createVisualSurfaces} joined it, keyed by index.
 * @param textures - The model's texture references, resolved or not.
 * @returns The located logical paths whose alpha is read by at least one surface.
 */
export function toAlphaTexturePaths(
  surfaces: ReadonlyMap<number, IRenderSurface>,
  textures: Array<VisualTextureDependency>
): ReadonlySet<string> {
  const paths: Set<string> = new Set();

  for (const texture of textures) {
    const surface: Nullable<IRenderSurface> = surfaces.get(texture.submeshIndex) ?? null;
    const asset: Nullable<XrayAsset> = getLocatedAsset(texture.resolution);

    if (asset && surface && isAlphaRenderSurface(surface)) {
      paths.add(asset.logicalPath);
    }
  }

  return paths;
}
