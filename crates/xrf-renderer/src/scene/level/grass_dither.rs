/// The engine's dither (`bwdithermap(2, dither)` over `magic4x4`), indexed `column * 16 + row` as the planting reads it:
/// `dither[(x + shift_x) % 16][(z + shift_z) % 16]`.
pub fn create_grass_dither() -> [u32; 256] {
  const MAGIC: [[u32; 4]; 4] = [[0, 14, 3, 13], [11, 5, 8, 6], [12, 2, 15, 1], [7, 9, 4, 10]];
  // `N = 255 / (levels - 1)` for two levels, and `magicfact = (N - 1) / 16`, single precision as the engine keeps it,
  // the sum then taken in double precision as its `16.` literal makes it.
  let factor: f64 = f64::from((255.0_f32 - 1.0) / 16.0);
  let mut dither: [u32; 256] = [0; 256];

  for i in 0..4 {
    for j in 0..4 {
      for k in 0..4 {
        for l in 0..4 {
          let value: f64 = 0.5 + f64::from(MAGIC[i][j]) * factor + (f64::from(MAGIC[k][l]) / 16.0) * factor;

          dither[(4 * k + i) * 16 + 4 * l + j] = value.trunc() as u32;
        }
      }
    }
  }

  dither
}
