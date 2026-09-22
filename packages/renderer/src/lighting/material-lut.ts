/** Columns: `dot(L, N)`, nearly linear, so few. */
export const MATERIAL_LUT_LDOTN: number = 128;
/** Rows: `dot(H, N)`, the specular lobe, so many. */
export const MATERIAL_LUT_LDOTH: number = 256;
/** Slices: the four lighting models a texture descriptor chooses between. */
export const MATERIAL_LUT_COUNT: number = 4;

/** The engine's `EPS_S` (`xrCore/math_constants.h`). */
const EPS_S: number = 0.0000001;

/**
 * The engine's `$user$material` lookup, built as `r4_rendertarget_build_textures.cpp` builds it.
 *
 * @returns `R8G8` texels, slice by slice, row by row.
 */
export function createMaterialLut(): Uint8Array {
  const data: Uint8Array = new Uint8Array(MATERIAL_LUT_LDOTN * MATERIAL_LUT_LDOTH * MATERIAL_LUT_COUNT * 2);

  for (let slice = 0; slice < MATERIAL_LUT_COUNT; slice++) {
    for (let y = 0; y < MATERIAL_LUT_LDOTH; y++) {
      for (let x = 0; x < MATERIAL_LUT_LDOTN; x++) {
        const at: number = ((slice * MATERIAL_LUT_LDOTH + y) * MATERIAL_LUT_LDOTN + x) * 2;
        const isCorner: boolean = y === MATERIAL_LUT_LDOTH - 1 && x === MATERIAL_LUT_LDOTN - 1;
        const [diffuse, specular] = isCorner ? [255, 255] : sampleMaterial(slice, x, y);

        data[at] = diffuse;
        data[at + 1] = specular;
      }
    }
  }

  return data;
}

/** One texel of one lighting model, as the engine quantises it. */
function sampleMaterial(slice: number, x: number, y: number): [number, number] {
  const ld: number = x / (MATERIAL_LUT_LDOTN - 1);
  const ls: number = (y / (MATERIAL_LUT_LDOTH - 1) + EPS_S) * Math.pow(ld, 1 / 32);
  let fd: number;
  let fs: number;

  switch (slice) {
    // Looks like Oren-Nayar.
    case 0:
      fd = Math.pow(ld, 0.75);
      fs = Math.pow(ls, 16) * 0.5;
      break;

    // Looks like Blinn.
    case 1:
      fd = Math.pow(ld, 0.9);
      fs = Math.pow(ls, 24);
      break;

    // Looks like Phong.
    case 2:
      fd = ld;
      fs = Math.pow(ls * 1.01, 128);
      break;

    // Looks like metal.
    default: {
      const s0: number = Math.abs(1 - Math.abs(0.05 * Math.sin(33 * ld) + ld - ls));
      const s1: number = Math.abs(1 - Math.abs(0.05 * Math.cos(33 * ld * ls) + ld - ls));
      const s2: number = Math.abs(1 - Math.abs(ld - ls));

      fd = ld;
      fs = Math.pow(Math.max(s0, s1, s2), 24) * Math.pow(ld, 1 / 7);
    }
  }

  return [quantise(fd), quantise(fs)];
}

/** `clampr(iFloor(value * 255.5f), 0, 255)`. */
function quantise(value: number): number {
  return Math.min(Math.max(Math.floor(value * 255.5), 0), 255);
}
