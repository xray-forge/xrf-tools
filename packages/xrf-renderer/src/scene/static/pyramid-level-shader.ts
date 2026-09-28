import { ComputeNode, UniformNode } from "three/webgpu";

/** A uniform the pyramid's builder sets per build. */
type TLevelUniform = UniformNode<"uint", number>;

/** One level of a depth pyramid being built, and the uniforms saying where it reads and writes. */
export interface IPyramidLevelShader {
  compute: ComputeNode;
  /** Where the level it reduces starts, and its width and height: the depth texture's size for the first level. */
  source: { offset: TLevelUniform; width: TLevelUniform; height: TLevelUniform };
  /** Where this level starts, and its width. */
  target: { offset: TLevelUniform; width: TLevelUniform };
}
