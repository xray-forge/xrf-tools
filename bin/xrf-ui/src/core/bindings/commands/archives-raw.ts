// Auto-generated rust bindings. Do not edit it manually.

import { AssetWorldSpec } from "@/core/bindings/types/xrf-app";
import { invokeRaw } from "@/core/ipc/raw";

/** Commands answering with raw bytes, which Specta cannot type. */
export const archivesRawCommands = {
  readImage: (world: AssetWorldSpec, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:archives|read_image", { world, logicalPath }),
};
