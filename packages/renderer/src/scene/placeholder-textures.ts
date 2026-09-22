import { Nullable } from "@xrf/types";
import { DataTexture, NoColorSpace, RGBAFormat, Texture, UnsignedByteType } from "three/webgpu";

let white: Nullable<Texture> = null;
let grey: Nullable<Texture> = null;

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

function createSolidTexture(value: number): Texture {
  const texture: DataTexture = new DataTexture(
    new Uint8Array([value, value, value, 255]),
    1,
    1,
    RGBAFormat,
    UnsignedByteType
  );

  texture.colorSpace = NoColorSpace;
  texture.needsUpdate = true;

  return texture;
}
