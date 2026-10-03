import { Nullable } from "@xrf/types";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ERenderPass, toRenderPass } from "@/core/render/lib/surface/render-pass";
import {
  IRenderSurfaceDraw,
  isWallmarkSurface,
  toRenderSurfaceDraw,
} from "@/core/render/lib/surface/render-surface-draw";

/**
 * Where in the frame a shader table entry is drawn, as a panel names it.
 *
 * @param descriptor - What the backend resolved for the entry, or null when none was declared or resolved.
 * @returns The pass, in words.
 */
export function describeLevelSurfacePass(descriptor: Nullable<XraySurfaceDescriptor>): string {
  const surface: IRenderSurfaceDraw = toRenderSurfaceDraw(descriptor);

  switch (toRenderPass(surface.draw, isWallmarkSurface(descriptor))) {
    case ERenderPass.DEFERRED:
      return "the G-buffer, lit by the sun and the hemisphere";

    case ERenderPass.WALLMARK:
      return "the albedo, before any light reaches it";

    case ERenderPass.FORWARD:
      return surface.isLit ? "over the lit frame, lit itself" : "over the lit frame, unlit";

    case ERenderPass.WATER:
      return "over the lit frame as water, reflecting the sky and distorting what is behind it";
  }
}
