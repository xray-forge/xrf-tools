import { ColorRepresentation, GridHelper, Group, Vector3 } from "three";

/** The mantissas a round step is allowed to take, so a grid is always countable in ones, twos or fives. */
const STEP_MANTISSAS: ReadonlyArray<number> = [1, 2, 5];

/**
 * How a ground grid is drawn, in colour and in density.
 */
export interface IRenderGridOptions {
  /** Colour of the ordinary lines. */
  color: number;
  /** Colour of the two lines crossing at the origin, which is the only thing marking where zero is. */
  originColor: number;
  /** Roughly how many cells to draw across the grid; the step is rounded off it, so the count is approximate. */
  cells: number;
}

/**
 * A round step near `extent / cells`, in whatever unit the extent is in.
 *
 * @param extent - How far the grid has to reach across.
 * @param cells - Roughly how many cells to cross it with.
 * @returns A step of one, two or five times a power of ten, never zero.
 */
export function toRenderGridStep(extent: number, cells: number): number {
  const wanted: number = Math.abs(extent) / Math.max(cells, 1);

  if (!Number.isFinite(wanted) || wanted <= 0) {
    return 1;
  }

  const magnitude: number = 10 ** Math.floor(Math.log10(wanted));
  const mantissa: number = STEP_MANTISSAS.find((it) => it >= wanted / magnitude) ?? 10;

  return mantissa * magnitude;
}

/**
 * The ground plane a scene is read against, with the origin marked on it.
 */
export class RenderGrid {
  /** Added to a scene by whoever owns it, and never by this. A group, so resizing it never re-parents anything. */
  public readonly object: Group = new Group();

  private readonly options: IRenderGridOptions;

  private grid: GridHelper;
  private gridStep: number = 1;

  public constructor(options: IRenderGridOptions) {
    this.options = options;
    this.grid = this.createGrid(this.gridStep);

    this.object.add(this.grid);
  }

  /** The distance one cell spans, for a caller sizing something else against it or saying what it measures in. */
  public get step(): number {
    return this.gridStep;
  }

  /**
   * Sizes the grid to what it is drawn under.
   *
   * @param extent - How far the grid has to reach from its centre, in the scene's own unit.
   */
  public setExtent(extent: number): void {
    const step: number = toRenderGridStep(extent * 2, this.options.cells);

    if (step === this.gridStep) {
      return;
    }

    this.gridStep = step;
    this.replaceGrid(this.createGrid(step));
  }

  /**
   * Moves the grid off the origin, for one marking something that is not there.
   *
   * @param center - Where the middle of the grid sits, in the scene's own coordinates.
   */
  public setCenter(center: Vector3): void {
    this.object.position.copy(center);
  }

  /**
   * @param isVisible - Whether the viewer is currently asking for it.
   */
  public setVisible(isVisible: boolean): void {
    // On the group rather than on the grid inside it, so a resize cannot bring back a grid somebody switched off.
    this.object.visible = isVisible;
  }

  /** Releases the geometry and material the helper owns. Whoever added `object` to a scene removes it. */
  public dispose(): void {
    this.grid.dispose();
  }

  /**
   * Rebuilt rather than scaled: a scaled grid keeps its line count, so a level would be crossed by the same
   * twenty-five lines a rifle is and its cells would stop being a round number of metres.
   */
  private replaceGrid(grid: GridHelper): void {
    this.object.remove(this.grid);
    this.grid.dispose();

    this.grid = grid;

    this.object.add(this.grid);
  }

  private createGrid(step: number): GridHelper {
    const grid: GridHelper = new GridHelper(
      step * this.options.cells,
      this.options.cells,
      this.options.originColor as ColorRepresentation,
      this.options.color as ColorRepresentation
    );

    // Drawn before everything solid and writing no depth: a grid is a backdrop for the geometry over it, and one
    // fighting the ground it lies on reads as an artefact.
    grid.renderOrder = -1;
    grid.material.depthWrite = false;

    return grid;
  }
}
