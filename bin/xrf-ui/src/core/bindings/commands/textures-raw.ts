// Auto-generated rust bindings. Do not edit it manually.

import { DocumentSessionId, TextureEncodingFormat } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { invokeRaw } from "@/core/ipc/raw";

/** Commands answering with raw bytes, which Specta cannot type. */
export const texturesRawCommands = {
  readCandidate: (sessionId: DocumentSessionId, format: TextureEncodingFormat): Promise<ArrayBuffer> =>
    invokeRaw("plugin:textures|read_candidate", { sessionId, format }),
  readTexture: (roots: XrayRoots, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:textures|read_texture", { roots, logicalPath }),
};
