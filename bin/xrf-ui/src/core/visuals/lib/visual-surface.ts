import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { XraySurfaceDescriptor, XraySurfaceDraw } from "@/core/ipc/types/xrf-material";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { VisualSubmesh, VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { Nullable } from "@/lib/types/general";

/**
 * The material state one submesh is drawn with, translated out of what the engine compiles for its shader.
 *
 * Three fields rather than the descriptor itself, because a renderer takes render states and not blender knobs: the
 * translation happens once, here, and the scene stays a place where materials are configured rather than a second
 * place where the engine is interpreted.
 */
export interface IVisualSurface {
  /**
   * Alpha below which a texel is discarded, `0` for a surface that reads no alpha.
   *
   * Three.js discards on `alpha < alphaTest`, which is the complement of `D3DCMP_GREATEREQUAL` against an alpha
   * reference, so the engine's own reference carries over unchanged.
   */
  alphaTest: number;
  /**
   * Whether the surface is composited over what is behind it rather than written opaquely.
   *
   * Only for the surfaces the engine takes out of its g-buffer: a cut-out is opaque everywhere it is not discarded,
   * and marking it transparent would sort it against the blended surfaces for nothing.
   */
  isTransparent: boolean;
  /**
   * Whether the surface writes depth.
   *
   * False exactly for the blended surfaces, which the engine draws with `zwrite` off
   * (`Layers/xrRender/blenders/blender_deffer_model.cpp`) so that two of them can be seen through one another.
   */
  isDepthWritten: boolean;
}

/** How a surface with nothing said about it is drawn, which is how the engine draws one whose shader it cannot find. */
export const OPAQUE_VISUAL_SURFACE: IVisualSurface = {
  alphaTest: 0,
  isDepthWritten: true,
  isTransparent: false,
};

/** The range an alpha reference is stated in on the wire, so `200` becomes `200 / 255`. */
const ALPHA_REFERENCE_SCALE: number = 255;

/**
 * Turn one resolved surface into the material state that draws it.
 *
 * @param descriptor - What the backend resolved for the submesh's shader name, or null when it declares none.
 * @returns The material state, opaque for anything the backend could not describe.
 */
export function toVisualSurface(descriptor: Nullable<XraySurfaceDescriptor>): IVisualSurface {
  const draw: Nullable<XraySurfaceDraw> = descriptor?.draw ?? null;

  if (!draw || draw.kind === "opaque") {
    return OPAQUE_VISUAL_SURFACE;
  }

  return {
    alphaTest: draw.reference / ALPHA_REFERENCE_SCALE,
    isDepthWritten: draw.kind !== "blended",
    isTransparent: draw.kind === "blended",
  };
}

/**
 * The material state of every submesh, by the index the submesh reports.
 *
 * @param submeshes - Submeshes as the backend described them.
 * @param surfaces - What the renderer draws for each declared shader name.
 * @returns One material state per submesh.
 */
export function createVisualSurfaces(
  submeshes: Array<VisualSubmesh>,
  surfaces: Record<string, XraySurfaceDescriptor> = {}
): Map<number, IVisualSurface> {
  return new Map(
    submeshes.map((submesh: VisualSubmesh) => [
      submesh.index,
      toVisualSurface(submesh.shaderName ? (surfaces[submesh.shaderName] ?? null) : null),
    ])
  );
}

/**
 * Whether a material state reads its texture's alpha channel at all.
 *
 * @param surface - Material state of one submesh.
 * @returns Whether anything is discarded or composited.
 */
export function isAlphaVisualSurface(surface: IVisualSurface): boolean {
  return surface.isTransparent || surface.alphaTest > 0;
}

/**
 * Logical paths of the texture files a model reads alpha out of.
 *
 * Needed because uploads are per **file** while alpha is per **surface**: a DXT1 file has to be uploaded in a format
 * that carries its one bit of alpha, and that decision belongs to the file rather than to each submesh naming it.
 * Answering "any submesh reads it" is sound in both directions - a submesh whose surface is opaque never samples
 * alpha, whatever format the file arrived in.
 *
 * @param surfaces - Material state per submesh index, as {@link createVisualSurfaces} joined it, keyed by index.
 * @param textures - The model's texture references, resolved or not.
 * @returns The located logical paths whose alpha is read by at least one surface.
 */
export function toAlphaTexturePaths(
  surfaces: ReadonlyMap<number, IVisualSurface>,
  textures: Array<VisualTextureDependency>
): ReadonlySet<string> {
  const paths: Set<string> = new Set();

  for (const texture of textures) {
    const surface: Nullable<IVisualSurface> = surfaces.get(texture.submeshIndex) ?? null;
    const asset: Nullable<XrayAsset> = getLocatedAsset(texture.resolution);

    if (asset && surface && isAlphaVisualSurface(surface)) {
      paths.add(asset.logicalPath);
    }
  }

  return paths;
}
