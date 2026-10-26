/**
 * A selected sprite cell as `[row, column]`.
 *
 * Row first, the way the lattice is indexed, while the configuration writes `inv_grid_x` before `inv_grid_y`. The
 * order is the one thing about a cell that can be got wrong silently, so it is stated wherever one is taken.
 */
export type TEquipmentCell = [number, number];
