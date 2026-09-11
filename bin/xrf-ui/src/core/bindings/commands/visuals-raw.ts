// Auto-generated rust bindings. Do not edit it manually.

import { DocumentSessionId } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { invokeRaw } from "@/core/ipc/raw";

/** Commands answering with raw bytes, which Specta cannot type. */
export const visualsRawCommands = {
  readGeometry: (sessionId: DocumentSessionId): Promise<ArrayBuffer> =>
    invokeRaw("plugin:visuals|read_geometry", { sessionId }),
  readMotion: (sessionId: DocumentSessionId, motionId: DocumentSessionId): Promise<ArrayBuffer> =>
    invokeRaw("plugin:visuals|read_motion", { sessionId, motionId }),
  readTexture: (roots: XrayRoots, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:visuals|read_texture", { roots, logicalPath }),
};
