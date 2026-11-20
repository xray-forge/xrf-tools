// Auto-generated rust bindings. Do not edit it manually.

/** The size of a picture a webview renders as it stands, read from its header alone. */
export type ImageShape = {
  width: number;
  height: number;
  /** The format as the decoder recognised it, which is what the bytes say rather than what the name claims. */
  format: string;
};

/** A format a texture can be written in, of the five worth offering. */
export enum ETextureEncodingFormat {
  BC1 = "bc1",
  BC2 = "bc2",
  BC3 = "bc3",
  RGBA8 = "rgba8",
  BC7 = "bc7",
}

/** Every `ETextureEncodingFormat` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureEncodingFormat = `${ETextureEncodingFormat}`;

/** How hard the encoder works, which trades seconds for fidelity. */
export enum ETextureEncodingQuality {
  FAST = "fast",
  NORMAL = "normal",
  /** The default, because everything but BC7 costs pennies at it. */
  SLOW = "slow",
}

/** Every `ETextureEncodingQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureEncodingQuality = `${ETextureEncodingQuality}`;
