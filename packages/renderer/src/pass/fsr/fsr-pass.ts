import { Nullable } from "@xrf/types";
import {
  ComputeNode,
  FloatType,
  NodeMaterial,
  QuadMesh,
  RedFormat,
  RenderTarget,
  RGFormat,
  StorageBufferAttribute,
  UnsignedByteType,
  WebGPURenderer,
} from "three/webgpu";

import { destroyStorageAttribute } from "#/internals/renderer-backend";
import { createColourTarget, IColourAttachment } from "#/pass/colour-target";
import { FrameCopyPass } from "#/pass/frame-copy-pass";
import { toFsrAccumulate } from "#/pass/fsr/fsr-accumulate.tsl";
import { IFsrInputs } from "#/pass/fsr/fsr-common.tsl";
import { toFsrDepthClip } from "#/pass/fsr/fsr-depth-clip.tsl";
import { toFsrLock } from "#/pass/fsr/fsr-lock.tsl";
import { LUMA_FIRST_STEP, toLumaFirstStep, toLumaShadingChange } from "#/pass/fsr/fsr-luminance-pyramid.tsl";
import { toFsrReactive } from "#/pass/fsr/fsr-reactive.tsl";
import {
  createFsrDepthClear,
  createFsrDepthReconstruction,
  toFsrDilate,
} from "#/pass/fsr/fsr-reconstruct-and-dilate.tsl";
import { PingPong } from "#/pass/ping-pong";
import { createDepthWritingQuadMaterial, createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { ResolvedTarget } from "#/pass/resolved-target";
import { ITemporalUpscaler } from "#/pass/temporal-upscaler";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { toUpscaledDepth } from "#/shader/drawn-sample.tsl";
import { FsrUniforms, toShadingChangeMipSide } from "#/uniforms/fsr-uniforms";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

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
  /** The history, the lock status and the luma history it writes, and the output. */
  accumulation: RenderTarget;
  accumulate: NodeMaterial;
  lock: NodeMaterial;
  /** Built with the reconstructed depth, which grows with the drawing. */
  clip: Nullable<NodeMaterial>;
}

/** The depth of the frame before as each texel's nearest depth reprojected puts it, and what writes it. */
interface IFsrReconstruction {
  depths: StorageBufferAttribute;
  capacity: number;
  clear: ComputeNode;
  reconstruct: ComputeNode;
}

/**
 * FSR 2.2's upscaler (FidelityFX, AMD, MIT) over the renderer's frame, which is in the display's range: an exposure of
 * one and no tonemap of its own, motion drawn at the render size.
 */
export class FsrPass implements ITemporalUpscaler {
  public readonly name: string = "fsr2";
  public readonly beforeBlended: ReadonlyArray<IRendererPass>;

  private readonly constants: FsrUniforms = new FsrUniforms();
  private readonly inputs: IFsrInputs;
  private readonly quad: QuadMesh = new QuadMesh();
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
  private readonly materials: {
    lumaFirst: NodeMaterial;
    lumaShading: NodeMaterial;
    dilate: NodeMaterial;
    reactive: NodeMaterial;
  };

  private reconstruction: Nullable<IFsrReconstruction> = null;
  private renderer: Nullable<WebGPURenderer> = null;
  private size: Nullable<IRendererFrameSize> = null;
  /** Frames resolved since the history was reset: FSR's `FrameIndex`. */
  private frameIndex: number = 0;

  /**
   * @param targets - The frame's targets, whose frame, depth and motion are upscaled.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.inputs = { color: targets.scene.texture, depth: targets.depth, motion: targets.motion };
    this.opaque = new FrameCopyPass("fsr2-opaque", targets.scene.texture);
    this.beforeBlended = [this.opaque];
    this.materials = {
      dilate: createQuadMaterial(toFsrDilate(this.inputs, this.constants)),
      lumaFirst: createQuadMaterial(toLumaFirstStep(this.inputs, this.constants)),
      lumaShading: createQuadMaterial(toLumaShadingChange(this.lumaFirst.texture)),
      reactive: createQuadMaterial(toFsrReactive(this.opaque.output.texture, this.inputs, this.constants)),
    };

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

    this.frames = new PingPong((index: 0 | 1) => {
      const previous: RenderTarget = accumulations[index === 0 ? 1 : 0];

      return {
        accumulate: createDepthWritingQuadMaterial(
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
        accumulation: accumulations[index],
        clip: null,
        dilate: dilates[index],
        lock: createQuadMaterial(toFsrLock(dilates[index].textures[2], this.constants)),
      };
    });
  }

  public get output(): RenderTarget {
    return this.resolved.output;
  }

  public resize(renderer: WebGPURenderer, size: IRendererFrameSize): void {
    const { renderWidth, renderHeight } = size;

    this.renderer = renderer;
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
    this.draw(renderer, this.materials.lumaFirst, this.lumaFirst);
    this.draw(renderer, this.materials.lumaShading, this.lumaShading);
    renderer.compute(reconstruction.reconstruct);
    this.draw(renderer, this.materials.dilate, frame.dilate);
    this.draw(renderer, this.materials.reactive, this.reactive);
    this.draw(renderer, frame.clip, this.clip);
    this.draw(renderer, frame.lock, this.locks);
    this.draw(renderer, frame.accumulate, frame.accumulation);

    this.frames.swap();
    this.frameIndex += 1;
  }

  public dispose(): void {
    // First, as it frees each frame's clip material and forgets it.
    this.release();
    this.opaque.dispose();
    [
      ...Object.values(this.materials),
      ...this.frames.both.flatMap(({ accumulate, lock, clip }: IFsrFrame) =>
        clip ? [accumulate, lock, clip] : [accumulate, lock]
      ),
    ].forEach((material: NodeMaterial) => material.dispose());
    [
      this.lumaFirst,
      this.lumaShading,
      this.reactive,
      this.clip,
      this.locks,
      ...this.frames.both.map((it) => it.dilate),
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

    this.reconstruction = {
      capacity: count,
      clear: createFsrDepthClear(depths, count),
      depths,
      reconstruct: createFsrDepthReconstruction(this.inputs, depths, count, this.constants),
    };
    this.frames.both.forEach((frame: IFsrFrame, index: number) => {
      const other: IFsrFrame = this.frames.both[index === 0 ? 1 : 0];

      frame.clip = createQuadMaterial(
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
      );
    });
  }

  /** Lets the reconstructed depth go, and the materials and computes built with it. */
  private release(): void {
    const { reconstruction, renderer } = this;

    if (!reconstruction) {
      return;
    }

    reconstruction.clear.dispose();
    reconstruction.reconstruct.dispose();
    this.frames.both.forEach((frame: IFsrFrame) => {
      frame.clip?.dispose();
      frame.clip = null;
    });

    if (renderer) {
      destroyStorageAttribute(renderer, reconstruction.depths);
    }

    this.reconstruction = null;
  }

  private draw(renderer: WebGPURenderer, material: NodeMaterial, target: RenderTarget): void {
    this.quad.material = material;
    renderer.setRenderTarget(target);
    this.quad.render(renderer);
  }
}
