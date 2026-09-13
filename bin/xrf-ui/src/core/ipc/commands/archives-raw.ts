// Auto-generated rust bindings. Do not edit it manually.

import { invokeRaw } from "@/core/ipc/raw";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands answering with raw bytes, which Specta cannot type. */
export const archivesRawCommands = {
  readImage: (roots: XrayRoots, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:archives|read_image", { roots, logicalPath }),
};
