import { AssetTextureDescriptor } from "@/core/ipc/types/xrf-app";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

/**
 * The short note beside an open texture's name: what the file is, in the terms a header has room for.
 *
 * @param descriptor - The base texture file, or null when its bytes could not be reached.
 * @returns What to print beside the name, or null when there is nothing to say.
 */
export function describeTextureCaption(descriptor: Nullable<AssetTextureDescriptor>): Nullable<string> {
  if (!descriptor) {
    return null;
  }

  const { shape, size } = descriptor;

  if (!shape) {
    return formatBytes(size);
  }

  return `${shape.width}×${shape.height} ${shape.format}`;
}
