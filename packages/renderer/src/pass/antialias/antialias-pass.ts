import { Nullable } from "@xrf/types";
import {
  HalfFloatType,
  LinearFilter,
  NearestFilter,
  NodeMaterial,
  QuadMesh,
  RenderTarget,
  Texture,
  WebGPURenderer,
} from "three/webgpu";

import { ERendererAntialiasing } from "#/contract/renderer-features";
import { createAntialiasSize, toFrameCopy, toFxaaStage, toSmaaPipeline } from "#/pass/antialias/antialias-stages.tsl";
import { SMAA_AREA_TEXTURE, SMAA_SEARCH_TEXTURE } from "#/pass/antialias/smaa-lookup";
import { ISmaaStages } from "#/pass/antialias/smaa-stages.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";

/** The target an antialiasing pass writes, or the frame itself where there is none. */
export function toPresentedFrame(pass: Nullable<AntialiasPass>, targets: RendererTargets): RenderTarget {
  return pass ? pass.output : targets.scene;
}

/** One stage of a mode: what it draws, and where. */
interface IAntialiasStage {
  material: NodeMaterial;
  target: RenderTarget;
}

/**
 * Smooths the finished frame's edges into a target of its own, which the frame then presents. In the frame only while a
 * mode is chosen, so no mode costs nothing; its targets and lookup textures go with it.
 */
export class AntialiasPass implements IRendererPass {
  public readonly name: string = "antialias";
  /** The smoothed frame. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false });

  private readonly quad: QuadMesh = new QuadMesh();
  private readonly source: RenderTarget;
  private readonly invSize = createAntialiasSize();
  private readonly stages: Array<IAntialiasStage>;
  private readonly targets: Array<RenderTarget> = [];
  private readonly lookups: Array<Texture> = [];
  /** A copy of the frame, drawn in place of the stages until what they sample has arrived. */
  private readonly copy: IAntialiasStage;
  private pending: number = 0;
  private isDisposed: boolean = false;

  /**
   * @param mode - How the edges are smoothed.
   * @param targets - The frame's targets, whose tonemapped frame is smoothed.
   */
  public constructor(mode: Exclude<ERendererAntialiasing, ERendererAntialiasing.NONE>, targets: RendererTargets) {
    this.source = targets.scene;
    this.output.texture.name = "antialiased";
    this.copy = { material: createQuadMaterial(toFrameCopy(this.source.texture)), target: this.output };
    this.stages = mode === ERendererAntialiasing.FXAA ? this.createFxaa() : this.createSmaa();
  }

  public render({ renderer }: IRendererFrame): void {
    const { width, height } = this.source;

    this.output.setSize(width, height);
    this.targets.forEach((target: RenderTarget) => target.setSize(width, height));
    this.invSize.value.set(1 / width, 1 / height);

    for (const stage of this.pending ? [this.copy] : this.stages) {
      this.draw(renderer, stage);
    }
  }

  public dispose(): void {
    this.isDisposed = true;
    [this.copy, ...this.stages].forEach((stage: IAntialiasStage) => stage.material.dispose());
    [this.output, ...this.targets].forEach((target: RenderTarget) => target.dispose());
    this.lookups.forEach((lookup: Texture) => lookup.dispose());
  }

  private draw(renderer: WebGPURenderer, stage: IAntialiasStage): void {
    this.quad.material = stage.material;
    renderer.setRenderTarget(stage.target);
    this.quad.render(renderer);
  }

  /** One stage, three's own `FXAANode` over the frame. */
  private createFxaa(): Array<IAntialiasStage> {
    return [{ material: createQuadMaterial(toFxaaStage(this.source.texture)), target: this.output }];
  }

  /** SMAA's three stages, the first two into targets of their own, as three's node draws them. */
  private createSmaa(): Array<IAntialiasStage> {
    const edges: RenderTarget = this.createStageTarget("smaa-edges");
    const weights: RenderTarget = this.createStageTarget("smaa-weights");
    const area: Texture = this.createLookup(SMAA_AREA_TEXTURE, LinearFilter, LinearFilter);
    const search: Texture = this.createLookup(SMAA_SEARCH_TEXTURE, NearestFilter, NearestFilter);
    const stages: ISmaaStages = toSmaaPipeline(
      { area, edges: edges.texture, frame: this.source.texture, search, weights: weights.texture },
      this.invSize
    );

    return [
      { material: createQuadMaterial(stages.edges), target: edges },
      { material: createQuadMaterial(stages.weights), target: weights },
      { material: createQuadMaterial(stages.blend), target: this.output },
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

        lookup.image = bitmap;
        lookup.needsUpdate = true;
        this.pending -= 1;
      })
      // Left pending, the frame is drawn unsmoothed rather than smoothed from nothing.
      .catch(() => {});

    return lookup;
  }
}
