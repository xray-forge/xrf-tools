// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { ETextureEncodingFormat, SessionId } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const texturesBulkRoutes = {
  readCandidate: (sessionId: SessionId, format: ETextureEncodingFormat): IBulkCall => ({
    args: { sessionId, format },
    route: "textures/read_candidate",
  }),
  readTexels: (roots: XrayRoots, logicalPath: string): IBulkCall => ({
    args: { roots, logicalPath },
    route: "textures/read_texels",
  }),
  readTexture: (roots: XrayRoots, logicalPath: string): IBulkCall => ({
    args: { roots, logicalPath },
    route: "textures/read_texture",
  }),
};
