/** What one row of the surface table says of a surface. */
export interface ISurfaceTableRow {
  tiling: number;
  detailScale: number;
  alphaReference: number;
  slice: number;
  color: readonly [number, number, number];
  /** Each slot's layer in the array its material samples it from, by the slot's place; zero for the rest. */
  layers: ReadonlyArray<number>;
}
