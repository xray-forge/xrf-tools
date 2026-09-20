import { AxesHelper, Object3D } from "three";

import { RenderGrid } from "@/core/render/lib/scene";

import { IVisualPreviewSceneConfig } from "./scene-config";
import { IVisualPreviewViewOptions } from "./visual-view-options";

/**
 * What a model is read against: the ground plane and the axis marker.
 */
export class VisualPreviewFrame {
  private readonly parent: Object3D;
  private readonly grid: RenderGrid;
  private readonly axes: AxesHelper = new AxesHelper(1);

  public constructor(parent: Object3D, config: IVisualPreviewSceneConfig) {
    this.parent = parent;
    this.grid = new RenderGrid({
      cells: config.gridCells,
      color: config.gridColor,
      originColor: config.gridOriginColor,
    });

    this.parent.add(this.grid.object);
    this.parent.add(this.axes);
  }

  /** The distance one cell of the grid spans, for anything sized against it. */
  public get gridStep(): number {
    return this.grid.step;
  }

  /**
   * Sizes the grid and the axis marker to what they are drawn around.
   *
   * @param radius - How far the model reaches, in the scene's own unit.
   */
  public setReach(radius: number): void {
    this.grid.setExtent(radius);
    this.axes.scale.setScalar(radius);
  }

  /**
   * @param options - What the toolbar has switched on.
   */
  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.grid.setVisible(options.isGridVisible);
    this.axes.visible = options.isAxesVisible;
  }

  /** Releases what the frame owns and takes it out of the scene. */
  public dispose(): void {
    this.parent.remove(this.grid.object);
    this.parent.remove(this.axes);

    this.grid.dispose();
    this.axes.dispose();
  }
}
