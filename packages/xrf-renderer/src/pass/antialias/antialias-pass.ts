import {
  HalfFloatType,
  LinearFilter,
  NearestFilter,
  RenderTarget,
  Texture,
  UniformNode,
  Vector2,
  WebGPURenderer,
} from "three/webgpu";

import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { TRendererSmoothingAntialiasing } from "#/contract/renderer-smoothing-antialiasing";
import { createAntialiasSize, toFxaaStage, toSmaaPipeline } from "#/pass/antialias/antialias-stages.tsl";
import { SMAA_AREA_TEXTURE, SMAA_SEARCH_TEXTURE } from "#/pass/antialias/smaa-lookup";
import { ISmaaStages } from "#/pass/antialias/smaa-stages";
import { toFrameCopy } from "#/pass/frame-copy-pass.tsl";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";

/**
 * Smooths the finished frame's edges into a target of its own, which the frame then presents. In the frame only while a
 * mode is chosen, so no mode costs nothing; its targets and lookup textures go with it.
 */
export class AntialiasPass implements IRendererPass {
  public readonly name: string = "antialias";
  /** The smoothed frame. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false });

  private readonly source: RenderTarget;
  private readonly invSize: UniformNode<"vec2", Vector2> = createAntialiasSize();
  /** The mode's stages, in order, each into its target. */
  private readonly stages: ReadonlyArray<FullScreenDraw>;
  private readonly targets: Array<RenderTarget> = [];
  private readonly lookups: Array<Texture> = [];
  /** The lookups' decoded pictures, which a texture never closes. */
  private readonly bitmaps: Array<ImageBitmap> = [];
  /** A copy of the frame, drawn in place of the stages until what they sample has arrived. */
  private readonly copy: FullScreenDraw;
  private pending: number = 0;
  private isDisposed: boolean = false;

  /**
   * @param mode - How the edges are smoothed.
   * @param targets - The frame's targets, whose tonemapped frame is smoothed.
   */
  public constructor(mode: TRendererSmoothingAntialiasing, targets: RendererTargets) {
    this.source = targets.scene;
    this.output.texture.name = "antialiased";
    this.copy = new FullScreenDraw(createQuadMaterial(toFrameCopy(this.source.texture)), this.output);
    this.stages = mode === ERendererAntialiasing.FXAA ? this.createFxaa() : this.createSmaa();
  }

  public resize(renderer: WebGPURenderer, { renderWidth, renderHeight }: IRendererFrameSize): void {
    [this.output, ...this.targets].forEach((target: RenderTarget) => {
      target.setSize(renderWidth, renderHeight);
      renderer.initRenderTarget(target);
    });
    this.invSize.value.set(1 / renderWidth, 1 / renderHeight);
  }

  /** The copy too, which draws until the lookups arrive. */
  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.copy);

    for (const stage of this.stages) {
      pipelines.draw(stage);
    }
  }

  public render({ renderer }: IRendererFrame): void {
    if (this.pending) {
      this.copy.render(renderer);

      return;
    }

    for (const stage of this.stages) {
      stage.render(renderer);
    }
  }

  public dispose(): void {
    this.isDisposed = true;
    [this.copy, ...this.stages].forEach((stage: FullScreenDraw) => stage.dispose());
    [this.output, ...this.targets].forEach((target: RenderTarget) => target.dispose());
    this.lookups.forEach((lookup: Texture) => lookup.dispose());
    this.bitmaps.forEach((bitmap: ImageBitmap) => bitmap.close());
  }

  /** One stage, three's own `FXAANode` over the frame. */
  private createFxaa(): Array<FullScreenDraw> {
    return [new FullScreenDraw(createQuadMaterial(toFxaaStage(this.source.texture)), this.output)];
  }

  /** SMAA's three stages, the first two into targets of their own, as three's node draws them. */
  private createSmaa(): Array<FullScreenDraw> {
    const edges: RenderTarget = this.createStageTarget("smaa-edges");
    const weights: RenderTarget = this.createStageTarget("smaa-weights");
    const area: Texture = this.createLookup(SMAA_AREA_TEXTURE, LinearFilter, LinearFilter);
    const search: Texture = this.createLookup(SMAA_SEARCH_TEXTURE, NearestFilter, NearestFilter);
    const stages: ISmaaStages = toSmaaPipeline(
      { area, edges: edges.texture, frame: this.source.texture, search, weights: weights.texture },
      this.invSize
    );

    return [
      new FullScreenDraw(createQuadMaterial(stages.edges), edges),
      new FullScreenDraw(createQuadMaterial(stages.weights), weights),
      new FullScreenDraw(createQuadMaterial(stages.blend), this.output),
    ];
  }

  private createStageTarget(name: string): RenderTarget {
    const target: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false, type: HalfFloatType });

    target.texture.name = name;
    this.targets.push(target);

    return target;
  }

  /**
   * A lookup texture from the data three carries it as, decoded off the page: a worker has no `Image`.
   *
   * @param uri - Its PNG, as a data URI.
   * @param minFilter - How it is sampled between texels, smaller.
   * @param magFilter - And larger.
   * @returns The texture, filled once the PNG is decoded.
   */
  private createLookup(
    uri: string,
    minFilter: typeof LinearFilter | typeof NearestFilter,
    magFilter: typeof LinearFilter | typeof NearestFilter
  ): Texture {
    const lookup: Texture = new Texture();

    lookup.minFilter = minFilter;
    lookup.magFilter = magFilter;
    lookup.generateMipmaps = false;
    lookup.flipY = false;
    this.lookups.push(lookup);
    this.pending += 1;

    fetch(uri)
      .then((response: Response) => response.blob())
      .then((blob: Blob) => createImageBitmap(blob, { colorSpaceConversion: "none", premultiplyAlpha: "none" }))
      .then((bitmap: ImageBitmap) => {
        if (this.isDisposed) {
          bitmap.close();

          return;
        }

        this.bitmaps.push(bitmap);
        lookup.image = bitmap;
        lookup.needsUpdate = true;
        this.pending -= 1;
      })
      // Left pending, the frame is drawn unsmoothed rather than smoothed from nothing.
      .catch((error: unknown) => console.error("SMAA's lookup failed to decode, so the frame is not smoothed:", error));

    return lookup;
  }
}
