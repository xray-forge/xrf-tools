import { RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { FullScreenDraw } from "#/pass/full-screen-draw";
import { PingPong } from "#/pass/ping-pong";
import { createDepthWritingQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { ResolvedTarget } from "#/pass/resolved-target";
import { toTemporalResolve } from "#/pass/temporal-antialias-pass.tsl";
import { ITemporalUpscaler } from "#/pass/temporal-upscaler";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { toUpscaledDepth } from "#/shader/drawn-sample.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { TemporalUniforms } from "#/uniforms/temporal-uniforms";

/**
 * TAA: the jittered frame resolved with its history, found through the motion every surface writes, at the output's
 * size, upscaling as TAAU where the scene is drawn smaller.
 */
export class TemporalAntialiasPass implements ITemporalUpscaler {
  public readonly name: string = "taa";
  public readonly beforeBlended: ReadonlyArray<IRendererPass> = [];

  private readonly resolved: ResolvedTarget = new ResolvedTarget("taa");
  private readonly temporal: TemporalUniforms = new TemporalUniforms();
  /** Each frame's resolve: into its history and the output, reading the other history. */
  private readonly stages: PingPong<FullScreenDraw>;

  /**
   * @param targets - The frame's targets, whose tonemapped frame is resolved.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    // Each frame's writer holds the history it writes; the other's is the frame before.
    const writers: ReadonlyArray<RenderTarget> = [0, 1].map((index: number) =>
      this.resolved.createWriter([{ name: `taa-history-${index}` }])
    );

    this.stages = new PingPong((index: 0 | 1) => {
      const history: Texture = writers[index === 0 ? 1 : 0].textures[0];

      return new FullScreenDraw(
        createDepthWritingQuadMaterial(
          toTemporalResolve(
            { depth: targets.depth, frame: targets.scene.texture, history, motion: targets.motion },
            { camera: uniforms.camera, motion: uniforms.motion, temporal: this.temporal }
          ),
          toUpscaledDepth(targets.scene.texture, targets.depth, uniforms.motion.jitter)
        ),
        writers[index]
      );
    });
  }

  public get output(): RenderTarget {
    return this.resolved.output;
  }

  public resize(renderer: WebGPURenderer, { width, height }: IRendererFrameSize): void {
    this.resolved.resize(renderer, width, height);
    this.resetHistory();
  }

  public resetHistory(): void {
    this.temporal.isHistoryValid.value = 0;
  }

  public listPipelines(pipelines: IRendererPipelines): void {
    for (const stage of this.stages.both) {
      pipelines.draw(stage);
    }
  }

  public render({ renderer }: IRendererFrame): void {
    this.stages.current.render(renderer);
    this.stages.swap();
    this.temporal.isHistoryValid.value = 1;
  }

  public dispose(): void {
    this.stages.both.forEach((stage: FullScreenDraw) => stage.dispose());
    this.resolved.dispose();
  }
}
