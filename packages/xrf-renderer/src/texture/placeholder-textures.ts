import { Nullable } from "@xrf/types";
import {
  DataArrayTexture,
  DataTexture,
  LinearFilter,
  NoColorSpace,
  RepeatWrapping,
  RGBA_S3TC_DXT1_Format,
  RGBAFormat,
  Texture,
  UnsignedByteType,
} from "three/webgpu";

import { DDS_CUBE_FACES } from "#/dds/dds-file";
import { createCubeTexture } from "#/texture/renderer-texture";
import { markRendererTextureNew } from "#/texture/renderer-texture-version";

let white: Nullable<Texture> = null;
let grey: Nullable<Texture> = null;
let flatBump: Nullable<Texture> = null;
let neutralDetailBump: Nullable<Texture> = null;
let neutralDetailBumpCompanion: Nullable<Texture> = null;
let flatBumpCompanion: Nullable<Texture> = null;
let sky: Nullable<Texture> = null;
let flatNormal: Nullable<Texture> = null;
let clear: Nullable<Texture> = null;
let array: Nullable<Texture> = null;

/**
 * @returns What an array slot's sampler is built with, which is what makes its shader declare an array: a layer of
 *   white. Every material drawing by the shader binds an array of its own.
 */
export function getPlaceholderArrayTexture(): Texture {
  if (!array) {
    const texture: DataArrayTexture = new DataArrayTexture(new Uint8Array([255, 255, 255, 255]), 1, 1, 1);

    texture.format = RGBAFormat;
    texture.type = UnsignedByteType;
    texture.colorSpace = NoColorSpace;
    // Filtered, as every array it stands in for is: three declares no sampler for a texture sampled nearest.
    texture.minFilter = LinearFilter;
    texture.magFilter = LinearFilter;
    markRendererTextureNew(texture);
    array = texture;
  }

  return array;
}

/**
 * @returns What a surface samples before its texture arrives, or without one: every channel one.
 */
export function getWhiteTexture(): Texture {
  return (white ??= createSolidTexture(255));
}

/**
 * @returns What a detail slot samples without its texture: a half that the engine's doubling cancels, in its colour
 *   and in the alpha that scales the gloss.
 */
export function getNeutralDetailTexture(): Texture {
  return (grey ??= createSolidTexture(128, 128, 128, 128));
}

/**
 * @returns What a detail's bump slot samples before its file arrives: a normal that, with its companion's, adds
 *   nothing, and a gloss the doubling leaves as it is.
 */
export function getNeutralDetailBumpTexture(): Texture {
  return (neutralDetailBump ??= createSolidTexture(128, 255, 128, 128));
}

/**
 * @returns What a detail's companion slot samples before its file arrives: what cancels the neutral detail bump.
 */
export function getNeutralDetailBumpCompanionTexture(): Texture {
  return (neutralDetailBumpCompanion ??= createSolidTexture(128, 128, 0, 0));
}

/**
 * @returns What a bump slot samples before its file arrives: a flat normal, `Nu.wzy` of one half-half-one, and
 * `def_gloss` in red, so a loading pair shades as though it were not there.
 */
export function getFlatBumpTexture(): Texture {
  return (flatBump ??= createSolidTexture(23, 255, 128, 128));
}

/**
 * @returns What a companion slot samples before its file arrives: no error, and no height.
 */
export function getFlatBumpCompanionTexture(): Texture {
  return (flatBumpCompanion ??= createSolidTexture(128, 128, 128, 0));
}

/**
 * @returns What a normal map samples before its file arrives: straight up in tangent space, a half, a half and one.
 */
export function getFlatNormalTexture(): Texture {
  return (flatNormal ??= createSolidTexture(128, 128, 255));
}

/**
 * @returns What an overlay samples before its file arrives, or without one: nothing, at no coverage.
 */
export function getClearTexture(): Texture {
  return (clear ??= createSolidTexture(0, 0, 0, 0));
}

/**
 * @returns What a sky cube samples before its file arrives: the mean of `default_clear`'s noon irradiance on every
 *   face, one block of each.
 */
export function getPlaceholderSkyTexture(): Texture {
  if (!sky) {
    // One BC1 block of a single 565 colour, both endpoints alike and every index the first.
    const color: number = (16 << 11) | (32 << 5) | 17;
    const block: Array<number> = [color & 0xff, color >> 8, color & 0xff, color >> 8, 0, 0, 0, 0];
    const data: Uint8Array = new Uint8Array(Array.from({ length: DDS_CUBE_FACES }, () => block).flat());

    sky = createCubeTexture({ height: 4, mipmaps: [{ data, height: 4, width: 4 }], width: 4 }, RGBA_S3TC_DXT1_Format);
  }

  return sky;
}

function createSolidTexture(red: number, green: number = red, blue: number = red, alpha: number = 255): Texture {
  const texture: DataTexture = new DataTexture(
    new Uint8Array([red, green, blue, alpha]),
    1,
    1,
    RGBAFormat,
    UnsignedByteType
  );

  texture.colorSpace = NoColorSpace;
  texture.magFilter = LinearFilter;
  texture.minFilter = LinearFilter;
  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;
  markRendererTextureNew(texture);

  return texture;
}
