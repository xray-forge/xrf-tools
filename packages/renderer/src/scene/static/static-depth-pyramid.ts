import { Nullable } from "@xrf/types";
import { ComputeNode, StorageBufferAttribute, Texture, WebGPURenderer } from "three/webgpu";

import { createPyramidShaders, IPyramidLevelShader, PYRAMID_REDUCTION } from "#/scene/static/static-occlusion.tsl";
import { EStaticPool, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * The depth a frame drew, reduced level by level to the farthest depth under each texel, for occlusion tests to read
 * a rectangle of the screen in four samples. Built into one buffer, all its levels in one compute pass; the buffer
 * grows to what a larger drawing needs.
 */
export class StaticDepthPyramid {
  private readonly buffers: StaticDrawBuffers;
  private shaders: Nullable<Array<IPyramidLevelShader>> = null;
  private depth: Nullable<Texture> = null;
  /** The buffer the shaders were built over. */
  private source: Nullable<StorageBufferAttribute> = null;
  private width: number = 0;
  private height: number = 0;
  private levels: Array<ComputeNode> = [];

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
  }

  /**
   * @param renderer - The renderer drawing.
   * @param depth - The depth texture as the frame has drawn it so far.
   * @param width - Its width.
   * @param height - Its height.
   * @returns Whether the pyramid was laid out afresh for a new size, which leaves any older one unreadable.
   */
  public build(renderer: WebGPURenderer, depth: Texture, width: number, height: number): boolean {
    if (width !== this.width || height !== this.height) {
      this.fit(width, height);
    }

    if (this.depth !== depth || this.source !== this.buffers.pyramid) {
      this.shaders?.forEach((shader: IPyramidLevelShader) => shader.compute.dispose());
      this.shaders = createPyramidShaders(this.buffers, depth);
      this.depth = depth;
      this.source = this.buffers.pyramid;
      this.width = 0;
    }

    const isLaidOut: boolean = width !== this.width || height !== this.height;

    if (isLaidOut) {
      this.layOut(width, height);
    }

    if (this.levels.length) {
      renderer.compute(this.levels);
    }

    return isLaidOut;
  }

  public dispose(): void {
    this.shaders?.forEach((shader: IPyramidLevelShader) => shader.compute.dispose());
    this.shaders = null;
  }

  /** Grows the buffer to hold every level a drawing of this size has, within its limit. */
  private fit(width: number, height: number): void {
    const { buffers } = this;
    const texels: number = StaticDepthPyramid.toLevels(width, height, buffers.occlusion.levels.length).reduce(
      (total: number, [levelWidth, levelHeight]) => total + levelWidth * levelHeight,
      0
    );

    if (texels > buffers.capacity(EStaticPool.PYRAMID)) {
      const capacity: number = Math.min(texels, buffers.limit(EStaticPool.PYRAMID));

      if (capacity > buffers.capacity(EStaticPool.PYRAMID)) {
        buffers.grow(EStaticPool.PYRAMID, capacity);
      }
    }
  }

  /**
   * @param width - The drawing's width.
   * @param height - Its height.
   * @param count - Levels at most.
   * @returns Each level's width and height: a quarter of the one before, to a single texel or `count` levels.
   */
  private static toLevels(width: number, height: number, count: number): Array<[number, number]> {
    const levels: Array<[number, number]> = [];
    let levelWidth: number = width;
    let levelHeight: number = height;

    while (levels.length < count && !(levelWidth === 1 && levelHeight === 1 && levels.length)) {
      levelWidth = Math.ceil(levelWidth / PYRAMID_REDUCTION);
      levelHeight = Math.ceil(levelHeight / PYRAMID_REDUCTION);
      levels.push([levelWidth, levelHeight]);
    }

    return levels;
  }

  /** Each level a quarter of the one before, to a single texel or as many levels as the tests read. */
  private layOut(width: number, height: number): void {
    const { occlusion } = this.buffers;
    const shaders: Array<IPyramidLevelShader> = this.shaders as Array<IPyramidLevelShader>;
    let offset: number = 0;
    let sourceWidth: number = width;
    let sourceHeight: number = height;
    let sourceOffset: number = 0;
    let span: number = PYRAMID_REDUCTION;

    this.width = width;
    this.height = height;
    this.levels = [];

    for (const [levelWidth, levelHeight] of StaticDepthPyramid.toLevels(width, height, shaders.length)) {
      const shader: IPyramidLevelShader = shaders[this.levels.length];

      // Levels past a buffer its limit kept short go untested: a test of a larger rectangle is skipped, not wrong.
      if (offset + levelWidth * levelHeight > this.buffers.capacity(EStaticPool.PYRAMID)) {
        break;
      }

      shader.source.offset.value = sourceOffset;
      shader.source.width.value = sourceWidth;
      shader.source.height.value = sourceHeight;
      shader.target.offset.value = offset;
      shader.target.width.value = levelWidth;
      shader.compute.count = levelWidth * levelHeight;
      occlusion.levels[this.levels.length].set(offset, levelWidth, levelHeight, span);
      this.levels.push(shader.compute);

      sourceOffset = offset;
      sourceWidth = levelWidth;
      sourceHeight = levelHeight;
      offset += levelWidth * levelHeight;
      span *= PYRAMID_REDUCTION;
    }

    occlusion.levelCount.value = this.levels.length;
    occlusion.size.value.set(width, height);
  }
}
