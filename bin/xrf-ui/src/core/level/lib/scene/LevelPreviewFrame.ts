import { AxesHelper, Box3, Box3Helper, Color, Object3D } from "three";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { toBoxFloor, toBoxReach, toLevelBox, toOriginReach } from "@/core/level/lib/extent/level-extent";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { RenderGrid } from "@/core/render/lib/scene/render-grid";
import { markThrough } from "@/core/render/lib/scene/render-marker";
import { Nullable } from "@/lib/types/general";

import { ILevelPreviewSceneConfig } from "./level-scene-config";

/** Which of the three the viewer is asking for; each answers where a level is in a different way. */
type ILevelFrameVisibility = Pick<ILevelViewOptions, "isAxesVisible" | "isBoundsVisible" | "isGridVisible">;

/** Cells of the grid the axis marker spans, so which way is which is legible without dwarfing the level. */
const AXES_CELLS: number = 2;

/**
 * What a level is read against: the ground plane, the origin, and the extent the level itself claims.
 */
export class LevelPreviewFrame {
  private readonly parent: Object3D;
  private readonly grid: RenderGrid;
  /** The same grid again over the level's own footprint, in the extent's colour, so its edge is legible. */
  private readonly boundsGrid: RenderGrid;
  private readonly axes: AxesHelper = new AxesHelper(1);
  private readonly extent: Box3Helper;

  /** The last extent taken, kept because whether the box can be shown is a property of it rather than of the toggle. */
  private box: Box3 = new Box3();
  private requested: ILevelFrameVisibility = { isAxesVisible: false, isBoundsVisible: false, isGridVisible: false };

  public constructor(parent: Object3D, config: ILevelPreviewSceneConfig) {
    this.parent = parent;
    this.grid = new RenderGrid({
      cells: config.gridCells,
      color: config.gridColor,
      originColor: config.gridOriginColor,
    });
    this.boundsGrid = new RenderGrid({
      cells: config.gridCells,
      color: config.boundsColor,
      originColor: config.boundsColor,
    });
    this.extent = new Box3Helper(this.box, new Color(config.boundsColor));

    // Never culled: the box outlines everything drawn, so a camera looking at any of it is inside the box, and
    // three.js would measure the helper against a frustum test it always fails.
    this.extent.frustumCulled = false;

    // The axis marker draws through the level; nothing else does. A marker is a point, so geometry hiding it hides
    // the answer; an extent is a whole box around everything drawn, so drawing it through the level puts four bright
    // lines across every view of it. Marsh puts its own origin under the terrain, which is what the marker is for.
    markThrough(this.axes);

    this.parent.add(this.grid.object);
    this.parent.add(this.boundsGrid.object);
    this.parent.add(this.axes);
    this.parent.add(this.extent);
    this.applyVisibility();
  }

  /** The distance one cell of the grid spans, so a viewer can say what it is measuring in. */
  public get gridStep(): number {
    return this.grid.step;
  }

  /**
   * Takes the level's measured extent, which sizes the grid and draws the box.
   *
   * @param bounds - What the backend measured, or null for a level that reports none and for no level at all.
   */
  public setBounds(bounds: Nullable<VisualBounds>): void {
    this.box = toLevelBox(bounds);
    this.extent.box = this.box;

    // Sized from the origin rather than from the box, because the grid is centred on the origin: a level lying a
    // kilometre off it needs a grid that reaches the level, not one the size of it.
    this.grid.setExtent(toOriginReach(this.box));
    // The other one is the level's own: centred on the box and reaching exactly as far as it does, so where the level
    // ends is a line rather than an inference from where the geometry stops.
    this.boundsGrid.setExtent(toBoxReach(this.box));
    this.boundsGrid.setCenter(toBoxFloor(this.box));
    this.axes.scale.setScalar(this.grid.step * AXES_CELLS);
    this.applyVisibility();
  }

  /**
   * @param options - What the toolbar has switched on.
   */
  public applyViewOptions(options: ILevelViewOptions): void {
    this.requested = options;

    this.applyVisibility();
  }

  /** Releases what the frame owns and takes it out of the scene. */
  public dispose(): void {
    this.parent.remove(this.grid.object);
    this.parent.remove(this.boundsGrid.object);
    this.parent.remove(this.axes);
    this.parent.remove(this.extent);

    this.grid.dispose();
    this.boundsGrid.dispose();
    this.axes.dispose();
    this.extent.dispose();
  }

  /** A box nothing was measured for is not drawn, however the toggle stands: it would be a dot at the origin. */
  private applyVisibility(): void {
    const hasExtent: boolean = this.requested.isBoundsVisible && !this.box.isEmpty();

    this.grid.setVisible(this.requested.isGridVisible);
    this.boundsGrid.setVisible(hasExtent);
    this.axes.visible = this.requested.isAxesVisible;
    this.extent.visible = hasExtent;
  }
}
