import { TextureNode, UniformNode, Vector2 } from "three/webgpu";

/** What the three stages read. */
export interface ISmaaInputs {
  /** The frame being smoothed. */
  sourceTexture: TextureNode;
  /** What the first stage wrote. */
  edgesTexture: TextureNode;
  /** What the second stage wrote. */
  weightsTexture: TextureNode;
  /** `AreaTex`, the precomputed blending areas. */
  areaTexture: TextureNode;
  /** `SearchTex`, the precomputed search lengths. */
  searchTexture: TextureNode;
  /** One over the frame's size in pixels. */
  invSize: UniformNode<"vec2", Vector2>;
}
