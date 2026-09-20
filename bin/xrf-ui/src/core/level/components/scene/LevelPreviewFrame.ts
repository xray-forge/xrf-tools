import { AxesHelper, Box3, Box3Helper, Color, Material, Object3D, Vector3 } from "three";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelPreviewSceneConfig } from "@/core/level/components/scene/level-scene-config";
import { ILevelViewOptions } from "@/core/level/lib/level-view-options";
import { RenderGrid } from "@/core/render/lib/render-grid";
import { Nullable } from "@/lib/types/general";

/** Which of the three the viewer is asking for; each answers where a level is in a different way. */
type ILevelFrameVisibility = Pick<ILevelViewOptions, "isAxesVisible" | "isBoundsVisible" | "isGridVisible">;

/** Cells of the grid the axis marker spans, so which way is which is legible without dwarfing the level. */
const AXES_CELLS: number = 2;

/** After everything the level draws, since a marker drawn through geometry has to be drawn after it too. */
const MARKER_RENDER_ORDER: number = 1;

/**
 * Draws a marker over the scene rather than in it, so geometry cannot hide the thing it is there to point at.
 *
 * Takes the material either way three types a helper's: the two here carry one each, and neither says so.
 */
function markThrough(marker: Object3D & { material: Material | Array<Material> }): void {
  for (const material of Array.isArray(marker.material) ? marker.material : [marker.material]) {
    material.depthTest = false;
  }

  marker.renderOrder = MARKER_RENDER_ORDER;
}

/** The measured extent as a box, empty for a level that reports none. */
function toBox(bounds: Nullable<VisualBounds>): Box3 {
  if (!bounds) {
    return new Box3();
  }

  const { min, max } = bounds.boundingBox;

  return new Box3(new Vector3(min.x ?? 0, min.y ?? 0, min.z ?? 0), new Vector3(max.x ?? 0, max.y ?? 0, max.z ?? 0));
}

/** How far a grid centred on a box has to reach to cover it, which is its longer horizontal half. */
function toBoxReach(box: Box3): number {
  if (box.isEmpty()) {
    return 1;
  }

  return Math.max(box.max.x - box.min.x, box.max.z - box.min.z) / 2;
}

/** The middle of a box's floor, which is where a grid marking it lies. */
function toBoxFloor(box: Box3): Vector3 {
  if (box.isEmpty()) {
    return new Vector3();
  }

  return new Vector3((box.min.x + box.max.x) / 2, box.min.y, (box.min.z + box.max.z) / 2);
}

/** How far the grid has to reach from the origin to cover a box, which is its furthest corner. */
function toOriginReach(box: Box3): number {
  if (box.isEmpty()) {
    return 1;
  }

  return Math.max(Math.abs(box.min.x), Math.abs(box.max.x), Math.abs(box.min.z), Math.abs(box.max.z));
}

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
    this.box = toBox(bounds);
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
