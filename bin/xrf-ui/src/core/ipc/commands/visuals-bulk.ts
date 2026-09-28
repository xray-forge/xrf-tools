// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { SessionId } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const visualsBulkRoutes = {
  readGeometry: (sessionId: SessionId): IBulkCall => ({ args: { sessionId }, route: "visuals/read_geometry" }),
  readMotion: (sessionId: SessionId, motionId: SessionId): IBulkCall => ({
    args: { sessionId, motionId },
    route: "visuals/read_motion",
  }),
  readTexture: (roots: XrayRoots, logicalPath: string): IBulkCall => ({
    args: { roots, logicalPath },
    route: "visuals/read_texture",
  }),
};
