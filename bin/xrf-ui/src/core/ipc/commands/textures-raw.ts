// Auto-generated rust bindings. Do not edit it manually.

import { invokeRaw } from "@/core/ipc/raw";
import { SessionId, TextureEncodingFormat } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands answering with raw bytes, which Specta cannot type. */
export const texturesRawCommands = {
  readCandidate: (sessionId: SessionId, format: TextureEncodingFormat): Promise<ArrayBuffer> =>
    invokeRaw("plugin:textures|read_candidate", { sessionId, format }),
  readTexture: (roots: XrayRoots, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:textures|read_texture", { roots, logicalPath }),
};
