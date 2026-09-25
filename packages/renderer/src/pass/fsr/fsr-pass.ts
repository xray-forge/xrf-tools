import { Nullable } from "@xrf/types";
import type { AnyPixelFormat, TextureDataType } from "three";
import {
  ComputeNode,
  DepthTexture,
  FloatType,
  HalfFloatType,
  LinearFilter,
  NearestFilter,
  NodeMaterial,
  PerspectiveCamera,
  QuadMesh,
  RedFormat,
  RenderTarget,
  RGBAFormat,
  RGFormat,
  StorageBufferAttribute,
  Texture,
  UnsignedByteType,
  Vector2,
  WebGPURenderer,
} from "three/webgpu";

import { initBorrowedDepthTarget } from "#/internals/borrowed-depth-target";
import { destroyStorageAttribute } from "#/internals/renderer-backend";
import { toFrameCopy } from "#/pass/antialias/antialias-stages.tsl";
import { toFsrAccumulate } from "#/pass/fsr/fsr-accumulate.tsl";
import {
  createFsrDepthClear,
  createFsrDepthReconstruction,
  IFsrInputs,
  toFsrDepthClip,
  toFsrDilate,
  toFsrLock,
  toFsrReactive,
  toLumaEighth,
  toLumaShadingChange,
} from "#/pass/fsr/fsr-prepare.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { TemporalJitter } from "#/pass/temporal-jitter";
import { ITemporalUpscaler } from "#/pass/temporal-upscaler";
import { toUpscaledDepth } from "#/pass/upscale-depth.tsl";
import { FsrUniforms } from "#/uniforms/fsr-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** One attachment of an FSR target: its format, type and whether it is sampled between texels. */
type TAttachment = readonly [AnyPixelFormat, TextureDataType, boolean];

const RED_HALF: TAttachment = [RedFormat, HalfFloatType, true];
const RG_HALF: TAttachment = [RGFormat, HalfFloatType, true];
const RGBA_HALF: TAttachment = [RGBAFormat, HalfFloatType, true];
const RED_BYTE: TAttachment = [RedFormat, UnsignedByteType, false];
const RGBA_BYTE: TAttachment = [RGBAFormat, UnsignedByteType, true];
const RED_FLOAT: TAttachment = [RedFormat, FloatType, false];

/** A target of one or more attachments, named for the device's labels. */
function createTarget(name: string, attachments: ReadonlyArray<TAttachment>): RenderTarget {
  const target: RenderTarget = new RenderTarget(1, 1, { count: attachments.length, depthBuffer: false });

  attachments.forEach(([format, type, isFiltered], index: number) => {
    const texture: Texture = target.textures[index];

    texture.name = attachments.length > 1 ? `${name}-${index}` : name;
    texture.format = format;
    texture.type = type;
    texture.minFilter = isFiltered ? LinearFilter : NearestFilter;
    texture.magFilter = isFiltered ? LinearFilter : NearestFilter;
    texture.generateMipmaps = false;
  });

  return target;
}

/** A quad drawing one material into one target. */
interface IFsrStage {
  material: NodeMaterial;
  target: RenderTarget;
}

/**
 * FSR 2.2's upscaler (FidelityFX, AMD, MIT), ported to the renderer's passes: the luminance the locks watch, the depth
 * of the frame before reconstructed by reprojection, the dilated depth and motion, a reactive mask from what the
 * blended surfaces changed, the depth clip and the reactive masks, the thin features to lock, and the accumulation at
 * the display's size. Its RCAS is the renderer's sharpening, with FSR 2's denoise. The frame's colour is the display's
 * already, so it runs with an exposure of one and no tonemap of its own; the motion is drawn at the render size.
 */
export class FsrPass implements ITemporalUpscaler {
  public readonly name: string = "fsr";
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true });
  /** A copy of the frame before the blended surfaces draw, which the reactive mask compares with. */
  public readonly opaque: IRendererPass;

  private readonly constants: FsrUniforms = new FsrUniforms();
  private readonly jitters: TemporalJitter;
  private readonly inputs: IFsrInputs;
  private readonly quad: QuadMesh = new QuadMesh();
  private readonly opaqueTarget: RenderTarget = createTarget("fsr-opaque", [RGBA_BYTE]);
  private readonly lumaEighth: RenderTarget = createTarget("fsr-luma-8", [RED_HALF]);
  private readonly lumaShading: RenderTarget = createTarget("fsr-luma-32", [RED_HALF]);
  /** Two, so the depth clip reads the dilated motion of the frame before. */
  private readonly dilates: ReadonlyArray<RenderTarget> = [0, 1].map((index: number) =>
    createTarget(`fsr-dilate-${index}`, [RED_FLOAT, RG_HALF, RED_HALF])
  );
  private readonly reactive: RenderTarget = createTarget("fsr-reactive", [RED_BYTE]);
  private readonly clip: RenderTarget = createTarget("fsr-clip", [RGBA_HALF, RG_HALF]);
  private readonly locks: RenderTarget = createTarget("fsr-locks", [RED_BYTE]);
  /** Two, each frame's accumulation reading the other's: history, lock status, luma history, and the output. */
  private readonly accumulations: ReadonlyArray<RenderTarget>;
  private readonly stages: {
    lumaEighth: IFsrStage;
    lumaShading: IFsrStage;
    reactive: IFsrStage;
    lock: ReadonlyArray<IFsrStage>;
    accumulate: ReadonlyArray<IFsrStage>;
  };
  private readonly dilate: NodeMaterial;
  private readonly accumulates: ReadonlyArray<NodeMaterial>;
  /** The reconstructed depth of the frame before, a `u32` a drawn texel, and what reads and writes it. */
  private depths: Nullable<StorageBufferAttribute> = null;
  private capacity: number = 0;
  private clear: Nullable<ComputeNode> = null;
  private reconstruct: Nullable<ComputeNode> = null;
  private clips: ReadonlyArray<NodeMaterial> = [];
  private renderer: Nullable<WebGPURenderer> = null;
  private readonly renderSize: Vector2 = new Vector2();
  private readonly displaySize: Vector2 = new Vector2();
  /** Which of each pair this frame writes. */
  private written: number = 0;
  private frameIndex: number = 0;

  /**
   * @param targets - The frame's targets, whose frame, depth and motion are upscaled.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    const outputTexture: Texture = createTarget("fsr-output", [RGBA_BYTE]).texture;
    const outputDepth: DepthTexture = new DepthTexture(1, 1, FloatType);

    outputDepth.name = "fsr-depth";
    this.output.texture.dispose();
    this.output.texture = outputTexture;
    this.output.depthTexture = outputDepth;
    this.jitters = new TemporalJitter(uniforms.motion);
    this.inputs = { color: targets.scene.texture, depth: targets.depth, motion: targets.motion };
    this.accumulations = [0, 1].map((index: number) => {
      const target: RenderTarget = createTarget(`fsr-accumulate-${index}`, [RGBA_HALF, RG_HALF, RGBA_BYTE, RGBA_BYTE]);

      target.textures[3].dispose();
      target.textures[3] = outputTexture;
      target.depthBuffer = true;
      target.depthTexture = outputDepth;

      return target;
    });
    this.dilate = createQuadMaterial(toFsrDilate(this.inputs, this.constants));
    this.accumulates = [0, 1].map((index: number) => {
      const previous: RenderTarget = this.accumulations[1 - index];
      const material: NodeMaterial = createQuadMaterial(
        toFsrAccumulate(
          {
            dilatedMotion: this.dilates[index].textures[1],
            frame: this.inputs.color,
            history: previous.textures[0],
            lockStatus: previous.textures[1],
            locks: this.locks.texture,
            lumaHistory: previous.textures[2],
            prepared: this.clip.textures[0],
            reactiveMasks: this.clip.textures[1],
            shadingLuma: this.lumaShading.texture,
          },
          this.constants,
          uniforms.motion.jitter
        )
      );

      // Written wherever it stands, the test left off: three turns `AlwaysDepth` into `NeverDepth` for reversed depth.
      material.depthNode = toUpscaledDepth(this.inputs.color, this.inputs.depth, uniforms.motion.jitter);
      material.depthWrite = true;

      return material;
    });
    this.stages = {
      accumulate: this.accumulates.map((material: NodeMaterial, index: number) => ({
        material,
        target: this.accumulations[index],
      })),
      lock: [
        { material: createQuadMaterial(toFsrLock(this.dilates[0].textures[2], this.constants)), target: this.locks },
        {
          material: createQuadMaterial(toFsrLock(this.dilates[1].textures[2], this.constants)),
          target: this.locks,
        },
      ],
      lumaEighth: { material: createQuadMaterial(toLumaEighth(this.inputs, this.constants)), target: this.lumaEighth },
      lumaShading: {
        material: createQuadMaterial(toLumaShadingChange(this.lumaEighth.texture)),
        target: this.lumaShading,
      },
      reactive: {
        material: createQuadMaterial(toFsrReactive(this.opaqueTarget.texture, this.inputs, this.constants)),
        target: this.reactive,
      },
    };

    const copy: NodeMaterial = createQuadMaterial(toFrameCopy(this.inputs.color));

    this.opaque = {
      dispose: (): void => copy.dispose(),
      name: "fsr-opaque",
      render: ({ renderer }: IRendererFrame): void =>
        this.draw(renderer, { material: copy, target: this.opaqueTarget }),
    };
  }

  public resize(
    renderer: WebGPURenderer,
    width: number,
    height: number,
    renderWidth: number,
    renderHeight: number
  ): void {
    if (
      width === this.displaySize.x &&
      height === this.displaySize.y &&
      renderWidth === this.renderSize.x &&
      renderHeight === this.renderSize.y
    ) {
      return;
    }

    this.renderer = renderer;
    this.displaySize.set(width, height);
    this.renderSize.set(renderWidth, renderHeight);
    this.jitters.resize(width / Math.max(renderWidth, 1), renderWidth, renderHeight);

    const mip: Vector2 = new Vector2(
      Math.max(1, Math.floor(renderWidth / 32)),
      Math.max(1, Math.floor(renderHeight / 32))
    );

    [this.opaqueTarget, ...this.dilates, this.reactive, this.clip, this.locks].forEach((target: RenderTarget) =>
      target.setSize(renderWidth, renderHeight)
    );
    this.lumaEighth.setSize(Math.ceil(renderWidth / 8), Math.ceil(renderHeight / 8));
    this.lumaShading.setSize(mip.x, mip.y);
    this.accumulations.forEach((target: RenderTarget) => target.setSize(width, height));
    this.output.setSize(width, height);
    // The accumulation writes the depth the helpers then draw over: none of the three may clear it on its first draw.
    this.accumulations.forEach((target: RenderTarget) => initBorrowedDepthTarget(renderer, target));
    initBorrowedDepthTarget(renderer, this.output);
    this.ensureCapacity(renderer, renderWidth * renderHeight);
    this.frameIndex = 0;
  }

  public jitter(camera: PerspectiveCamera): void {
    this.jitters.apply(camera);
  }

  public render({ renderer, camera }: IRendererFrame): void {
    if (!this.clear || !this.reconstruct) {
      return;
    }

    const current: number = this.written;
    const count: number = this.renderSize.x * this.renderSize.y;

    this.constants.follow(camera, this.renderSize, this.displaySize, this.jitters.offset, this.jitters.phaseCount);
    this.constants.frameIndex.value = this.frameIndex;
    this.clear.count = count;
    this.reconstruct.count = count;

    renderer.compute(this.clear);
    this.draw(renderer, this.stages.lumaEighth);
    this.draw(renderer, this.stages.lumaShading);
    renderer.compute(this.reconstruct);
    this.draw(renderer, { material: this.dilate, target: this.dilates[current] });
    this.draw(renderer, this.stages.reactive);
    this.draw(renderer, { material: this.clips[current], target: this.clip });
    this.draw(renderer, this.stages.lock[current]);
    this.draw(renderer, this.stages.accumulate[current]);

    this.written = 1 - current;
    this.frameIndex += 1;
    this.jitters.advance();
    this.jitters.restore();
  }

  public dispose(): void {
    this.jitters.dispose();
    this.opaque.dispose();
    [
      this.dilate,
      ...this.accumulates,
      ...this.clips,
      this.stages.lumaEighth.material,
      this.stages.lumaShading.material,
      this.stages.reactive.material,
      ...this.stages.lock.map((stage: IFsrStage) => stage.material),
    ].forEach((material: NodeMaterial) => material.dispose());
    [
      this.opaqueTarget,
      this.lumaEighth,
      this.lumaShading,
      ...this.dilates,
      this.reactive,
      this.clip,
      this.locks,
      ...this.accumulations,
      this.output,
    ].forEach((target: RenderTarget) => target.dispose());
    this.output.texture.dispose();
    this.output.depthTexture?.dispose();
    this.clear?.dispose();
    this.reconstruct?.dispose();

    if (this.renderer && this.depths) {
      destroyStorageAttribute(this.renderer, this.depths);
    }
  }

  /** Grows the reconstructed depth to a drawing's texels, and what reads and writes it with it. */
  private ensureCapacity(renderer: WebGPURenderer, count: number): void {
    if (count <= this.capacity) {
      return;
    }

    if (this.depths) {
      destroyStorageAttribute(renderer, this.depths);
    }

    this.clear?.dispose();
    this.reconstruct?.dispose();
    this.clips.forEach((material: NodeMaterial) => material.dispose());
    this.capacity = count;
    this.depths = new StorageBufferAttribute(new Uint32Array(count), 1);
    this.clear = createFsrDepthClear(this.depths, count);
    this.reconstruct = createFsrDepthReconstruction(this.inputs, this.depths, count, this.constants);
    this.clips = [0, 1].map((index: number) =>
      createQuadMaterial(
        toFsrDepthClip(
          this.inputs,
          {
            capacity: count,
            dilatedDepth: this.dilates[index].textures[0],
            dilatedMotion: this.dilates[index].textures[1],
            previousDilatedMotion: this.dilates[1 - index].textures[1],
            reactive: this.reactive.texture,
            reconstructed: this.depths as StorageBufferAttribute,
          },
          this.constants
        )
      )
    );
  }

  private draw(renderer: WebGPURenderer, stage: IFsrStage): void {
    this.quad.material = stage.material;
    renderer.setRenderTarget(stage.target);
    this.quad.render(renderer);
  }
}
