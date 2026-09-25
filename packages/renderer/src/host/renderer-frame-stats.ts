import { Vector2 } from "three/webgpu";

import { IRendererCameraPose } from "#/contract/renderer-camera";
import { IRendererLightsReport, IRendererReport, IRendererStaticDrawReport } from "#/contract/renderer-report";
import { RendererDevice } from "#/device/renderer-device";
import { RenderFrameTimer } from "#/frame/render-frame-timer";
import { RendererGpuTimings } from "#/host/renderer-gpu-timings";
import { IStaticCullCounts } from "#/scene/static/static-cull-counts";

/** How often the frame report is sent, in milliseconds. */
const REPORT_INTERVAL: number = 250;

/**
 * What the frames cost, on the CPU and the GPU, and when it is next reported.
 */
export class RendererFrameStats {
  private readonly frameTimer: RenderFrameTimer = new RenderFrameTimer();
  private readonly gpuTimings: RendererGpuTimings = new RendererGpuTimings();
  private reportedAt: number = 0;

  /**
   * @param now - When the frame about to be drawn began.
   */
  public beginFrame(now: number): void {
    this.frameTimer.sample(now);
  }

  /**
   * @param elapsed - Milliseconds the frame's passes took to submit.
   * @param device - The device that drew it.
   */
  public endFrame(elapsed: number, device: RendererDevice): void {
    this.frameTimer.sampleDraw(elapsed);
    this.gpuTimings.resolve(device);
  }

  /**
   * @param now - The time now.
   * @returns Whether a report is due, counting it sent if so.
   */
  public takeReport(now: number): boolean {
    if (now - this.reportedAt < REPORT_INTERVAL) {
      return false;
    }

    this.reportedAt = now;

    return true;
  }

  /**
   * @param device - The device drawing.
   * @param canvas - The canvas drawn on.
   * @param rendered - The scene's size as drawn.
   * @param camera - Where the camera stands.
   * @param passes - The frame's passes, in frame order.
   * @param kept - What the static cull kept, which three's own counts leave out.
   * @param staticDraws - How full the static draws' pools are and what occlusion removed.
   * @returns What the frames have been costing.
   */
  public toReport(
    device: RendererDevice,
    canvas: OffscreenCanvas,
    rendered: Vector2,
    camera: IRendererCameraPose,
    passes: ReadonlyArray<string>,
    kept: IStaticCullCounts,
    staticDraws: IRendererStaticDrawReport,
    lights: IRendererLightsReport
  ): IRendererReport {
    const { render } = device.renderer.info;

    return {
      camera,
      frame: {
        drawnHeight: canvas.height,
        drawnWidth: canvas.width,
        drawTime: this.frameTimer.drawTime,
        draws: render.drawCalls + kept.draws,
        frameTime: this.frameTimer.frameTime,
        framesPerSecond: this.frameTimer.framesPerSecond,
        renderedHeight: rendered.y,
        renderedWidth: rendered.x,
        triangles: render.triangles + kept.triangles,
        worstDrawTime: this.frameTimer.worstDrawTime,
        worstFrameTime: this.frameTimer.worstFrameTime,
      },
      isGpuTimed: device.isTiming,
      lights,
      passes: this.gpuTimings.describe(passes),
      staticDraws,
    };
  }

  /** Forgets the frames timed, for a view shown again after a gap nothing drew in. */
  public restart(): void {
    this.frameTimer.reset();
  }

  /** Forgets every pass's timing, for timing that stopped or started again: a mean over the gap would lie. */
  public resetTimings(): void {
    this.gpuTimings.reset();
  }

  /** Forgets everything, for a device that went away. */
  public reset(): void {
    this.frameTimer.reset();
    this.gpuTimings.reset();
    this.reportedAt = 0;
  }
}
