// Auto-generated rust bindings. Do not edit it manually.

import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { invokeRaw } from "@/core/ipc/raw";

/** Commands answering with raw bytes, which Specta cannot type. */
export const archivesRawCommands = {
  readImage: (roots: XrayRoots, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:archives|read_image", { roots, logicalPath }),
};
