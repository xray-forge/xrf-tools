/**
 * The engine's dither (`bwdithermap(2, dither)` over `magic4x4`), indexed `column * 16 + row` as the planting reads
 * it: `dither[(x + shift_x) % 16][(z + shift_z) % 16]`.
 *
 * @returns Its 256 thresholds.
 */
export function createGrassDither(): Uint32Array {
  const magic: ReadonlyArray<ReadonlyArray<number>> = [
    [0, 14, 3, 13],
    [11, 5, 8, 6],
    [12, 2, 15, 1],
    [7, 9, 4, 10],
  ];
  // `N = 255 / (levels - 1)` for two levels, and `magicfact = (N - 1) / 16`, in single precision as the engine has it.
  const factor: number = Math.fround(Math.fround(255 - 1) / 16);
  const dither: Uint32Array = new Uint32Array(256);

  for (let i = 0; i < 4; i++) {
    for (let j = 0; j < 4; j++) {
      for (let k = 0; k < 4; k++) {
        for (let l = 0; l < 4; l++) {
          dither[(4 * k + i) * 16 + 4 * l + j] = Math.trunc(0.5 + magic[i][j] * factor + (magic[k][l] / 16) * factor);
        }
      }
    }
  }

  return dither;
}
