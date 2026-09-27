import { Nullable } from "@xrf/types";
import {
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

let white: Nullable<Texture> = null;
let grey: Nullable<Texture> = null;
let flatBump: Nullable<Texture> = null;
let flatBumpCompanion: Nullable<Texture> = null;
let sky: Nullable<Texture> = null;
let flatNormal: Nullable<Texture> = null;
let clear: Nullable<Texture> = null;

/**
 * @returns What a surface samples before its texture arrives, or without one: every channel one.
 */
export function getWhiteTexture(): Texture {
  return (white ??= createSolidTexture(255));
}

/**
 * @returns What a detail slot samples without its texture: a half that the engine's doubling cancels.
 */
export function getNeutralDetailTexture(): Texture {
  return (grey ??= createSolidTexture(128));
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
  // One BC1 block of a single 565 colour, both endpoints alike and every index the first.
  const color: number = (16 << 11) | (32 << 5) | 17;
  const block: Array<number> = [color & 0xff, color >> 8, color & 0xff, color >> 8, 0, 0, 0, 0];
  const data: Uint8Array = new Uint8Array(Array.from({ length: DDS_CUBE_FACES }, () => block).flat());

  return (sky ??= createCubeTexture(
    { height: 4, mipmaps: [{ data, height: 4, width: 4 }], width: 4 },
    RGBA_S3TC_DXT1_Format
  ));
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
  texture.needsUpdate = true;

  return texture;
}
