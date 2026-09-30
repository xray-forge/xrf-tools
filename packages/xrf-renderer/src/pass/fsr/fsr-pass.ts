import { Nullable } from "@xrf/types";
import {
  ComputeNode,
  FloatType,
  RedFormat,
  RenderTarget,
  RGFormat,
  StorageBufferAttribute,
  UnsignedByteType,
  WebGPURenderer,
} from "three/webgpu";

import { IColourAttachment } from "#/pass/colour-attachment";
import { createColourTarget } from "#/pass/colour-target";
import { FrameCopyPass } from "#/pass/frame-copy-pass";
import { toFsrAccumulate } from "#/pass/fsr/fsr-accumulate.tsl";
import { toFsrDepthClip } from "#/pass/fsr/fsr-depth-clip.tsl";
import { IFsrInputs } from "#/pass/fsr/fsr-inputs";
import { toFsrLock } from "#/pass/fsr/fsr-lock.tsl";
import { LUMA_FIRST_STEP, toLumaFirstStep, toLumaShadingChange } from "#/pass/fsr/fsr-luminance-pyramid.tsl";
import { toFsrReactive } from "#/pass/fsr/fsr-reactive.tsl";
import {
  createFsrDepthClear,
  createFsrDepthReconstruction,
  toFsrDilate,
} from "#/pass/fsr/fsr-reconstruct-and-dilate.tsl";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { PingPong } from "#/pass/ping-pong";
import { createDepthWritingQuadMaterial, createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { ResolvedTarget } from "#/pass/resolved-target";
import { ITemporalUpscaler } from "#/pass/temporal-upscaler";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { toUpscaledDepth } from "#/shader/drawn-sample.tsl";
import { FsrUniforms, toShadingChangeMipSide } from "#/uniforms/fsr-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { StorageRetirement } from "#/uniforms/storage-retirement";

/** One channel of half floats. */
const RED_HALF: Omit<IColourAttachment, "name"> = { format: RedFormat };
/** Two channels of half floats. */
const RG_HALF: Omit<IColourAttachment, "name"> = { format: RGFormat };
/** One channel of bytes, read texel by texel. */
const RED_BYTE: Omit<IColourAttachment, "name"> = { format: RedFormat, isFiltered: false, type: UnsignedByteType };

/** What one frame of the two writes, reading the other's. */
interface IFsrFrame {
  /** The dilated depth, the dilated motion, and the lock luma. */
  dilate: RenderTarget;
  /** Into the history, the lock status and the luma history, and the output. */
  accumulate: FullScreenDraw;
  lock: FullScreenDraw;
  /** Built with the reconstructed depth, which grows with the drawing. */
  clip: Nullable<FullScreenDraw>;
}

/** The depth of the frame before as each texel's nearest depth reprojected puts it, and what writes it. */
interface IFsrReconstruction {
  depths: StorageBufferAttribute;
  capacity: number;
  clear: ComputeNode;
  reconstruct: ComputeNode;
  /** Both, as the pass names them. */
  kernels: ReadonlyArray<ComputeNode>;
}

/**
 * FSR 2.2's upscaler (FidelityFX, AMD, MIT) over the renderer's frame, which is in the display's range: an exposure of
 * one and no tonemap of its own, motion drawn at the render size.
 */
export class FsrPass implements ITemporalUpscaler {
  public readonly name: string = "fsr2";
  public readonly beforeBlended: ReadonlyArray<IRendererPass>;

  private readonly constants: FsrUniforms = new FsrUniforms();
  private readonly retirement: StorageRetirement;
  private readonly inputs: IFsrInputs;
  /** The frame before the blended surfaces drew, which the reactive mask compares with. */
  private readonly opaque: FrameCopyPass;
  private readonly lumaFirst: RenderTarget = createColourTarget([{ name: "fsr2-luma-8", ...RED_HALF }]);
  private readonly lumaShading: RenderTarget = createColourTarget([{ name: "fsr2-luma-32", ...RED_HALF }]);
  private readonly reactive: RenderTarget = createColourTarget([{ name: "fsr2-reactive", ...RED_BYTE }]);
  private readonly clip: RenderTarget = createColourTarget([
    { name: "fsr2-prepared" },
    { name: "fsr2-masks", ...RG_HALF },
  ]);
  private readonly locks: RenderTarget = createColourTarget([{ name: "fsr2-locks", ...RED_BYTE }]);
  private readonly resolved: ResolvedTarget = new ResolvedTarget("fsr2");
  private readonly frames: PingPong<IFsrFrame>;
  private readonly draws: {
    lumaFirst: FullScreenDraw;
    lumaShading: FullScreenDraw;
    /** Into either frame's dilate, which it is drawn into by turns. */
    dilate: FullScreenDraw;
    reactive: FullScreenDraw;
  };

  private reconstruction: Nullable<IFsrReconstruction> = null;
  private size: Nullable<IRendererFrameSize> = null;
  /** Frames resolved since the history was reset: FSR's `FrameIndex`. */
  private frameIndex: number = 0;

  /**
   * @param targets - The frame's targets, whose frame, depth and motion are upscaled.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.retirement = uniforms.retirement;
    this.inputs = { color: targets.scene.texture, depth: targets.depth, motion: targets.motion };
    this.opaque = new FrameCopyPass("fsr2-opaque", targets.scene.texture);
    this.beforeBlended = [this.opaque];

    // Each frame's targets first, since each frame's materials read the other's.
    const dilates: ReadonlyArray<RenderTarget> = [0, 1].map((index: number) =>
      createColourTarget([
        { format: RedFormat, isFiltered: false, name: `fsr2-dilated-depth-${index}`, type: FloatType },
        { name: `fsr2-dilated-motion-${index}`, ...RG_HALF },
        { name: `fsr2-lock-luma-${index}`, ...RED_HALF },
      ])
    );
    const accumulations: ReadonlyArray<RenderTarget> = [0, 1].map((index: number) =>
      this.resolved.createWriter([
        { name: `fsr2-history-${index}` },
        { name: `fsr2-lock-status-${index}`, ...RG_HALF },
        { name: `fsr2-luma-history-${index}`, type: UnsignedByteType },
      ])
    );

    this.draws = {
      dilate: new FullScreenDraw(createQuadMaterial(toFsrDilate(this.inputs, this.constants)), dilates[0]),
      lumaFirst: new FullScreenDraw(createQuadMaterial(toLumaFirstStep(this.inputs, this.constants)), this.lumaFirst),
      lumaShading: new FullScreenDraw(
        createQuadMaterial(toLumaShadingChange(this.lumaFirst.texture)),
        this.lumaShading
      ),
      reactive: new FullScreenDraw(
        createQuadMaterial(toFsrReactive(this.opaque.output.texture, this.inputs, this.constants)),
        this.reactive
      ),
    };
    this.frames = new PingPong((index: 0 | 1) => {
      const previous: RenderTarget = accumulations[index === 0 ? 1 : 0];

      return {
        accumulate: new FullScreenDraw(
          createDepthWritingQuadMaterial(
            toFsrAccumulate(
              {
                dilatedMotion: dilates[index].textures[1],
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
            ),
            toUpscaledDepth(this.inputs.color, this.inputs.depth, uniforms.motion.jitter)
          ),
          accumulations[index]
        ),
        clip: null,
        dilate: dilates[index],
        lock: new FullScreenDraw(createQuadMaterial(toFsrLock(dilates[index].textures[2], this.constants)), this.locks),
      };
    });
  }

  public get output(): RenderTarget {
    return this.resolved.output;
  }

  public resize(renderer: WebGPURenderer, size: IRendererFrameSize): void {
    const { renderWidth, renderHeight } = size;

    this.size = size;

    for (const target of [
      this.reactive,
      this.clip,
      this.locks,
      ...this.frames.both.map((it: IFsrFrame) => it.dilate),
    ]) {
      target.setSize(renderWidth, renderHeight);
      renderer.initRenderTarget(target);
    }

    this.lumaFirst.setSize(Math.ceil(renderWidth / LUMA_FIRST_STEP), Math.ceil(renderHeight / LUMA_FIRST_STEP));
    this.lumaShading.setSize(toShadingChangeMipSide(renderWidth), toShadingChangeMipSide(renderHeight));
    [this.lumaFirst, this.lumaShading].forEach((target: RenderTarget) => renderer.initRenderTarget(target));
    this.resolved.resize(renderer, size.width, size.height);
    this.reserve(renderWidth * renderHeight);
    this.resetHistory();
  }

  public resetHistory(): void {
    this.frameIndex = 0;
  }

  /** Both frames', and what the reconstructed depth is built with once the pass is sized. */
  public listPipelines(pipelines: IRendererPipelines): void {
    const { draws, reconstruction } = this;

    pipelines.draw(draws.lumaFirst);
    pipelines.draw(draws.lumaShading);
    pipelines.draw(draws.dilate);
    pipelines.draw(draws.reactive);

    for (const { accumulate, clip, lock } of this.frames.both) {
      pipelines.draw(accumulate);
      pipelines.draw(lock);

      if (clip) {
        pipelines.draw(clip);
      }
    }

    if (reconstruction) {
      pipelines.compute(reconstruction.kernels);
    }
  }

  public render({ renderer, camera, jitter }: IRendererFrame): void {
    const { reconstruction, size } = this;
    const frame: IFsrFrame = this.frames.current;

    if (!reconstruction || !size || !jitter || !frame.clip) {
      return;
    }

    this.constants.take(camera, size, jitter, this.frameIndex);
    reconstruction.clear.count = size.renderWidth * size.renderHeight;
    reconstruction.reconstruct.count = reconstruction.clear.count;

    renderer.compute(reconstruction.clear);
    this.draws.lumaFirst.render(renderer);
    this.draws.lumaShading.render(renderer);
    renderer.compute(reconstruction.reconstruct);
    this.draws.dilate.render(renderer, frame.dilate);
    this.draws.reactive.render(renderer);
    frame.clip.render(renderer);
    frame.lock.render(renderer);
    frame.accumulate.render(renderer);

    this.frames.swap();
    this.frameIndex += 1;
  }

  public dispose(): void {
    // First, as it frees each frame's clip material and forgets it.
    this.release();
    this.opaque.dispose();
    [
      ...Object.values(this.draws),
      ...this.frames.both.flatMap(({ accumulate, lock }: IFsrFrame) => [accumulate, lock]),
    ].forEach((draw: FullScreenDraw) => draw.dispose());
    [
      this.lumaFirst,
      this.lumaShading,
      this.reactive,
      this.clip,
      this.locks,
      ...this.frames.both.map((it: IFsrFrame) => it.dilate),
    ].forEach((target: RenderTarget) => target.dispose());
    this.resolved.dispose();
  }

  /** Grows the reconstructed depth to hold a drawing's texels, and what reads and writes it with it. */
  private reserve(count: number): void {
    if (this.reconstruction && count <= this.reconstruction.capacity) {
      return;
    }

    this.release();

    const depths: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(count), 1);
    const clear: ComputeNode = createFsrDepthClear(depths, count);
    const reconstruct: ComputeNode = createFsrDepthReconstruction(this.inputs, depths, count, this.constants);

    // Cleared and written on the GPU alone.
    this.retirement.retireZeroed([depths]);

    this.reconstruction = { capacity: count, clear, depths, kernels: [clear, reconstruct], reconstruct };
    this.frames.both.forEach((frame: IFsrFrame, index: number) => {
      const other: IFsrFrame = this.frames.both[index === 0 ? 1 : 0];

      frame.clip = new FullScreenDraw(
        createQuadMaterial(
          toFsrDepthClip(
            this.inputs,
            {
              capacity: count,
              dilatedDepth: frame.dilate.textures[0],
              dilatedMotion: frame.dilate.textures[1],
              previousDilatedMotion: other.dilate.textures[1],
              reactive: this.reactive.texture,
              reconstructed: depths,
            },
            this.constants
          )
        ),
        this.clip
      );
    });
  }

  /** Lets the reconstructed depth go once no frame binds it, and the materials and computes built with it. */
  private release(): void {
    const { reconstruction } = this;

    if (!reconstruction) {
      return;
    }

    reconstruction.clear.dispose();
    reconstruction.reconstruct.dispose();
    this.frames.both.forEach((frame: IFsrFrame) => {
      frame.clip?.dispose();
      frame.clip = null;
    });

    this.retirement.retire([reconstruction.depths]);
    this.reconstruction = null;
  }
}
