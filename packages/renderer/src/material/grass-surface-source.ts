import { Node, StorageBufferNode } from "three/webgpu";

import { IRendererSurface } from "#/contract/scene/renderer-surface";

/** What one model's grass is drawn from. */
export interface IGrassSurfaceSource {
  surface: IRendererSurface;
  /** The model's bounding box height, which a vertex's share of the sway is measured against. */
  height: number;
  /** Every planted tuft, sorted by model: two vectors each, its place and turn, then its scale, light and wave. */
  items: StorageBufferNode<"vec4">;
  /** Where the model's tufts start among the items. */
  start: Node<"uint">;
}
