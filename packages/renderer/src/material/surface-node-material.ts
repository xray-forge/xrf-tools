import { Nullable } from "@xrf/types";
import { MeshBasicNodeMaterial, Node, NodeBuilder, NodeMaterialObserver } from "three/webgpu";

import { StaticDrawObserver } from "#/material/static-draw-observer";
import { isBufferPlacedBuild, toBufferPlacedPositionView } from "#/shader/placement.tsl";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { TreeWindUniforms } from "#/uniforms/tree-wind-uniforms";

/**
 * The material every surface draws with: three's basic node material, placing a static draw by the buffers every
 * static draw shares rather than by its object's matrices, and refreshing it only when its bundle records.
 */
export class SurfaceNodeMaterial extends MeshBasicNodeMaterial {
  private readonly staticDraws: StaticDrawBuffers;
  private readonly wind: TreeWindUniforms;

  /**
   * @param staticDraws - What static draws are placed by.
   * @param wind - How the trees sway, which a tree drawn statically takes.
   */
  public constructor(staticDraws: StaticDrawBuffers, wind: TreeWindUniforms) {
    super();
    this.staticDraws = staticDraws;
    this.wind = wind;
  }

  /** Where a vertex stands in view space, for a surface placing its own vertices; null for every other. */
  public positionViewNode: Nullable<Node> = null;

  public override setupPositionView(builder: NodeBuilder): Node {
    if (this.positionViewNode) {
      return this.positionViewNode;
    }

    return isBufferPlacedBuild(builder)
      ? toBufferPlacedPositionView(builder, this.staticDraws, this.wind)
      : super.setupPositionView(builder);
  }

  public override setupObserver(builder: NodeBuilder): NodeMaterialObserver {
    return isBufferPlacedBuild(builder) ? new StaticDrawObserver(builder) : super.setupObserver(builder);
  }
}
