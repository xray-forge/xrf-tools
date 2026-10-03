import { EMPTY_RENDER_FRAME_COST, IRendererPassTimings, IRenderFrameCost } from "@xrf/renderer";

import { RenderFrameReport, RenderPassCost } from "@/core/ipc/types/xrf-renderer";

/**
 * @param report - What a native viewport's recent frames cost.
 * @returns The same, as the readouts count a frame: the static batches' indirect draws its draw calls.
 */
export function toNativeFrameCost(report: RenderFrameReport): IRenderFrameCost {
  const cpuTime: number = report.cpuTime ?? 0;

  return {
    ...EMPTY_RENDER_FRAME_COST,
    draws: report.staticDraws.commands,
    drawnHeight: report.height,
    drawnWidth: report.width,
    drawTime: cpuTime,
    frameTime: report.frameTime ?? 0,
    framesPerSecond: report.framesPerSecond ?? 0,
    renderedHeight: report.renderHeight,
    renderedWidth: report.renderWidth,
    triangles: report.triangles,
    worstDrawTime: cpuTime,
    worstFrameTime: report.frameTimeMax ?? 0,
  };
}

/**
 * @param report - What a native viewport's recent frames cost.
 * @returns What each of its passes cost on the GPU, as the readout lists them.
 */
export function toNativePassTimings(report: RenderFrameReport): IRendererPassTimings {
  return {
    isGpuTimed: report.isGpuTimed,
    passes: report.passes.map((pass: RenderPassCost) => ({ gpuTime: pass.gpuTime ?? 0, name: pass.name })),
  };
}
