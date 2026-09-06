import { describeTextureShape, getLocatedAsset } from "@/core/assets/lib";
import { AssetTextureDescriptor, TextureDescription } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { formatBytes } from "@/lib/memory/format";
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
  const { bump } = description.material;

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

/**
 * One file in a line: where it came from, how large it is, and what layout it declares.
 *
 * @param file - The half being described.
 * @returns The line for its row.
 */
export function describeTextureFile(file: ITextureFile): string {
  if (!file.asset) {
    return "Not bound";
  }

  const parts: Array<string> = [file.asset.logicalPath, describeContainer(file.asset)];

  if (file.descriptor) {
    parts.push(formatBytes(file.descriptor.size));
    // A file whose header will not parse is exactly the one worth knowing the size of, so the size still shows.
    parts.push(file.descriptor.shape ? describeTextureShape(file.descriptor.shape) : "header unreadable");
  }

  return parts.join(" · ");
}

/** Where a located file physically sits, since a loose file and an archived entry are edited very differently. */
function describeContainer(asset: XrayAsset): string {
  return asset.container.kind === "archive" ? `in ${asset.container.path}` : `in ${asset.container.root}`;
}
