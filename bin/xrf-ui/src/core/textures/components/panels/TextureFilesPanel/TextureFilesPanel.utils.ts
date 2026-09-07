import { getLocatedAsset } from "@/core/assets/lib";
import { AssetTextureDescriptor, TextureDescription } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { Nullable } from "@/lib/types/general";

/** One of the three files a bumped surface is drawn from, as far as this root set holds it. */
export interface ITextureFile {
  label: string;
  asset: Nullable<XrayAsset>;
  descriptor: Nullable<AssetTextureDescriptor>;
}

/**
 * The three files behind a texture, named by what the engine ends up binding.
 *
 * @param description - The texture as the backend resolved it.
 * @returns The base texture and both halves of its pair, in the order a person reads them.
 */
export function selectBoundTextureFiles(description: TextureDescription): Array<ITextureFile> {
  const bump = description.material?.bump ?? null;

  return [
    { asset: description.texture, descriptor: description.base, label: "Texture" },
    { asset: bump ? getLocatedAsset(bump.bump.resolution) : null, descriptor: description.bump, label: "Bump" },
    {
      asset: bump ? getLocatedAsset(bump.companion.resolution) : null,
      descriptor: description.companion,
      label: "Bump#",
    },
  ];
}
