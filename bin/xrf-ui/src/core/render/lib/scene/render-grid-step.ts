/** The mantissas a round step is allowed to take, so a grid is always countable in ones, twos or fives. */
const STEP_MANTISSAS: ReadonlyArray<number> = [1, 2, 5];

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
