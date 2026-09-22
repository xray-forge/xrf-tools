import { Nullable } from "@xrf/types";

import { AssetTextureShape, TextureDescription } from "@/core/ipc/types/xrf-app";

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

  status.push(description.form ? "has descriptor" : "no descriptor");

  return status;
}
