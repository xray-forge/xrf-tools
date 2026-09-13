// Auto-generated rust bindings. Do not edit it manually.

import { invokeRaw } from "@/core/ipc/raw";
import { SessionId } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands answering with raw bytes, which Specta cannot type. */
export const visualsRawCommands = {
  readGeometry: (sessionId: SessionId): Promise<ArrayBuffer> =>
    invokeRaw("plugin:visuals|read_geometry", { sessionId }),
  readMotion: (sessionId: SessionId, motionId: SessionId): Promise<ArrayBuffer> =>
    invokeRaw("plugin:visuals|read_motion", { sessionId, motionId }),
  readTexture: (roots: XrayRoots, logicalPath: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:visuals|read_texture", { roots, logicalPath }),
};
