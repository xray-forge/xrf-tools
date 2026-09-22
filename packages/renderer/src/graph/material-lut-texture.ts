import { ClampToEdgeWrapping, Data3DTexture, LinearFilter, RGFormat, UnsignedByteType } from "three/webgpu";

import { createMaterialLut, MATERIAL_LUT_COUNT, MATERIAL_LUT_LDOTH, MATERIAL_LUT_LDOTN } from "#/lighting/material-lut";

/**
 * The engine's material lookup on the GPU: `R8G8`, linear, clamped, as `$user$material` is bound.
 *
 * @returns The texture, uploaded on first use.
 */
export function createMaterialLutTexture(): Data3DTexture {
  const texture: Data3DTexture = new Data3DTexture(
    createMaterialLut(),
    MATERIAL_LUT_LDOTN,
    MATERIAL_LUT_LDOTH,
    MATERIAL_LUT_COUNT
  );

  texture.format = RGFormat;
  texture.type = UnsignedByteType;
  texture.minFilter = LinearFilter;
  texture.magFilter = LinearFilter;
  texture.wrapS = ClampToEdgeWrapping;
  texture.wrapT = ClampToEdgeWrapping;
  texture.wrapR = ClampToEdgeWrapping;
  texture.generateMipmaps = false;
  texture.needsUpdate = true;

  return texture;
}
