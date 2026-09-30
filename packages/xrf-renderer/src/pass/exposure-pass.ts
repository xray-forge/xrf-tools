import { Nullable } from "@xrf/types";
import { ComputeNode, WebGPURenderer } from "three/webgpu";

import { IRendererExposureSettings } from "#/contract/renderer-exposure-settings";
import { createExposureAdaptation, createExposureMeasure } from "#/pass/exposure-pass.tsl";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
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
  /** Both, measured first, as they dispatch. */
  private readonly kernels: ReadonlyArray<ComputeNode>;
  private adaptation: number = 1;
  /** The frame's time it last adapted at, in seconds, or null before it adapted at all. */
  private last: Nullable<number> = null;

  /**
   * @param targets - What the frame draws into, the frame combine writes among them.
   * @param exposure - The adapted scale, and the constants it adapts by.
   */
  public constructor(targets: RendererTargets, exposure: ExposureUniforms) {
    this.exposure = exposure;
    this.measure = createExposureMeasure(targets.scene.texture, exposure, exposure.size);
    this.adapt = createExposureAdaptation(exposure);
    this.kernels = [this.measure, this.adapt];
  }

  /**
   * @param settings - How the exposure adapts.
   */
  public setSettings(settings: IRendererExposureSettings): void {
    this.exposure.apply(settings);
    this.adaptation = settings.adaptation;
  }

  public resize(_renderer: WebGPURenderer, { renderWidth, renderHeight }: IRendererFrameSize): void {
    this.exposure.size.value.set(renderWidth, renderHeight);
  }

  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.compute(this.kernels);
  }

  public render({ renderer, time }: IRendererFrame): void {
    const delta: number = this.last === null ? 0 : Math.min(time - this.last, LONGEST_STEP);

    this.last = time;
    this.exposure.advance(delta, this.adaptation);
    renderer.compute([this.measure, this.adapt]);
  }

  /** Leaves the frame at the noon answer again, and lets three forget both computes. */
  public dispose(): void {
    this.measure.dispose();
    this.adapt.dispose();
    this.exposure.reset();
  }
}
