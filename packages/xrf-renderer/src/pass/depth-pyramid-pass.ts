import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCull } from "#/scene/static/static-cull";

/**
 * Reduces the depth the static draws finished with, before any plain draw, for the next frame's first cull to read.
 */
export class DepthPyramidPass implements IRendererPass {
  public readonly name: string = "pyramid";

  private readonly targets: RendererTargets;
  private readonly cull: StaticCull;

  /**
   * @param targets - What the frame draws into, whose depth is reduced.
   * @param cull - What culls the static draws, which reads the pyramid.
   */
  public constructor(targets: RendererTargets, cull: StaticCull) {
    this.targets = targets;
    this.cull = cull;
  }

  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.compute(this.cull.getPyramidKernels(this.targets.depth));
  }

  public render({ renderer }: IRendererFrame): void {
    this.cull.finish(renderer, this.targets.depth, this.targets.gbuffer.width, this.targets.gbuffer.height);
  }

  public dispose(): void {}
}
