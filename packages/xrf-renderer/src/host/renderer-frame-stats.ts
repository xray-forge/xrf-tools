import { IRendererReport } from "#/contract/renderer-report";
import { RendererDevice } from "#/device/renderer-device";
import { RenderFrameTimer } from "#/frame/render-frame-timer";
import { RendererGpuTimings } from "#/host/renderer-gpu-timings";
import { IRendererReportInput } from "#/host/renderer-report-input";

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
   * @param input - What the report is taken from.
   * @returns What the frames have been costing.
   */
  public toReport(input: IRendererReportInput): IRendererReport {
    const { device, canvas, size, camera, passes, kept, staticDraws, lights, cpuMemory, weather } = input;
    const { render } = device.renderer.info;

    return {
      camera,
      cpuMemory,
      frame: {
        drawnHeight: canvas.height,
        drawnWidth: canvas.width,
        drawTime: this.frameTimer.drawTime,
        draws: render.drawCalls + staticDraws.commands,
        frameTime: this.frameTimer.frameTime,
        framesPerSecond: this.frameTimer.framesPerSecond,
        renderedHeight: size.renderHeight,
        renderedWidth: size.renderWidth,
        triangles: render.triangles + kept.triangles,
        worstDrawTime: this.frameTimer.worstDrawTime,
        worstFrameTime: this.frameTimer.worstFrameTime,
      },
      isGpuTimed: device.isTiming,
      lights,
      passes: this.gpuTimings.describe(passes),
      staticDraws,
      weather,
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
}
