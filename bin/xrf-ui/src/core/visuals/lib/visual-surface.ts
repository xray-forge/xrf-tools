import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualSubmesh } from "@/core/ipc/types/xrf-visual";
import { getRenderSurfaceDraw, IRenderSurfaceDraw } from "@/core/render/lib/surface/render-surface-draw";

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
): Map<number, IRenderSurfaceDraw> {
  return new Map(
    submeshes.map((submesh: VisualSubmesh, position: number) => [
      submesh.index,
      getRenderSurfaceDraw(surfaces, position),
    ])
  );
}
