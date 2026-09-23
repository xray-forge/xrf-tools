import { Nullable } from "@xrf/types";
import { DataTexture, NoColorSpace, RGBAFormat, Texture, UnsignedByteType } from "three/webgpu";

let white: Nullable<Texture> = null;
let grey: Nullable<Texture> = null;
let flatBump: Nullable<Texture> = null;
let flatBumpCompanion: Nullable<Texture> = null;

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

function createSolidTexture(red: number, green: number = red, blue: number = red, alpha: number = 255): Texture {
  const texture: DataTexture = new DataTexture(
    new Uint8Array([red, green, blue, alpha]),
    1,
    1,
    RGBAFormat,
    UnsignedByteType
  );

  texture.colorSpace = NoColorSpace;
  texture.needsUpdate = true;

  return texture;
}
