// Auto-generated rust bindings. Do not edit it manually.

/**
 * The format both halves of a bump pair are written in.
 *
 * Only formats that keep alpha at full precision are offered, because the pair stores the two values that must stay
 * precise there: the normal's `x` in the first half and the height in the second. BC1 keeps one bit of alpha and BC2
 * four. The error plane of the second half is measured against whichever of these encoded the first half, so the two
 * halves always share one format.
 */
export enum EGenerateBumpFormat {
  /** DXT5, the format `DXTCompressBump` writes, which DX9 and DX11 both load. */
  BC3 = "bc3",
  /** BC7, which loses less of the normals but which D3DX9, and so the DX9 renderer, cannot load. */
  BC7 = "bc7",
}

/** Every `EGenerateBumpFormat` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type GenerateBumpFormat = `${EGenerateBumpFormat}`;

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
