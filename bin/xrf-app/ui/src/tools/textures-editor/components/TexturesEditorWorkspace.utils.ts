import { AssetTextureShape } from "@xrf/ipc/types/xrf-app-core";
import { TextureDescription } from "@xrf/ipc/types/xrf-app-textures";
import { Nullable } from "@xrf/types";

/**
 * What the status bar says about the texture being worked on.
 *
 * @param description - The open texture, or null before one is.
 * @returns The status entries.
 */
export function describeEditedTextureStatus(description: Nullable<TextureDescription>): Array<string> {
  if (!description) {
    return [];
  }

  const status: Array<string> = [description.reference];
  const shape: Nullable<AssetTextureShape> = description.base?.shape ?? null;

  if (shape) {
    status.push(`${shape.width}x${shape.height}`, shape.format);
  }

  status.push(description.hasDescriptor ? "has descriptor" : "no descriptor");

  return status;
}
