// Auto-generated rust bindings. Do not edit it manually.

import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { invokeRaw } from "@/core/ipc/raw";

/** Commands answering with raw bytes, which Specta cannot type. */
export const texturesRawCommands = {
  readTexture: (roots: XrayRoots, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:textures|read_texture", { roots, logicalPath }),
};
