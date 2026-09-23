import { Nullable } from "@xrf/types";
import { ComputeNode, Texture, WebGPURenderer } from "three/webgpu";

import { createPyramidShaders, IPyramidLevelShader, PYRAMID_REDUCTION } from "#/scene/static/static-occlusion.tsl";
import { STATIC_PYRAMID_CAPACITY, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * The depth a frame drew, reduced level by level to the farthest depth under each texel, for occlusion tests to read
 * a rectangle of the screen in four samples. Built into one buffer, all its levels in one compute pass.
 */
export class StaticDepthPyramid {
  private readonly buffers: StaticDrawBuffers;
  private shaders: Nullable<Array<IPyramidLevelShader>> = null;
  private depth: Nullable<Texture> = null;
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
    if (this.depth !== depth) {
      this.shaders?.forEach((shader: IPyramidLevelShader) => shader.compute.dispose());
      this.shaders = createPyramidShaders(this.buffers, depth);
      this.depth = depth;
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

    for (const shader of shaders) {
      const levelWidth: number = Math.ceil(sourceWidth / PYRAMID_REDUCTION);
      const levelHeight: number = Math.ceil(sourceHeight / PYRAMID_REDUCTION);

      if (offset + levelWidth * levelHeight > STATIC_PYRAMID_CAPACITY) {
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

      if (levelWidth === 1 && levelHeight === 1) {
        break;
      }
    }

    occlusion.levelCount.value = this.levels.length;
    occlusion.size.value.set(width, height);
  }
}
