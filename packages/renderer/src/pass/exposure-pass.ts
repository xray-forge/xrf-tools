import { ComputeNode } from "three/webgpu";

import { IRendererExposureSettings } from "#/contract/renderer-features";
import { createExposureAdaptation, createExposureMeasure } from "#/pass/exposure-pass.tsl";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { ExposureUniforms } from "#/uniforms/exposure-uniforms";

/** The longest step between frames the adaptation takes, so a stall does not snap it to what one frame measured. */
const LONGEST_STEP: number = 0.25;

/**
 * `phase_luminance`: the frame combine finished measured, and the scale every later tonemap multiplies by adapted
 * towards it on the GPU, read by the next frame as the engine's `s_tonemap` is.
 */
export class ExposurePass implements IRendererPass {
  public readonly name: string = "exposure";

  private readonly exposure: ExposureUniforms;
  private readonly measure: ComputeNode;
  private readonly adapt: ComputeNode;
  private adaptation: number = 1;
  private last: number = -1;

  /**
   * @param targets - What the frame draws into, the frame combine writes among them.
   * @param exposure - The adapted scale, and the constants it adapts by.
   */
  public constructor(targets: RendererTargets, exposure: ExposureUniforms) {
    this.exposure = exposure;
    this.measure = createExposureMeasure(targets.scene.texture, exposure, exposure.size);
    this.adapt = createExposureAdaptation(exposure);
  }

  /**
   * @param settings - How the exposure adapts.
   */
  public setSettings(settings: IRendererExposureSettings): void {
    this.exposure.apply(settings);
    this.adaptation = settings.adaptation;
  }

  public resize(_: unknown, { renderWidth, renderHeight }: IRendererFrameSize): void {
    this.exposure.size.value.set(renderWidth, renderHeight);
  }

  public render({ renderer }: IRendererFrame): void {
    const now: number = performance.now() / 1000;
    const delta: number = this.last < 0 ? 0 : Math.min(now - this.last, LONGEST_STEP);

    this.last = now;
    this.exposure.advance(delta, this.adaptation);
    renderer.compute([this.measure, this.adapt]);
  }

  /** Leaves the frame at the noon answer again. */
  public dispose(): void {
    this.exposure.reset();
  }
}
