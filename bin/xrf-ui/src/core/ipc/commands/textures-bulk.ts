// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { AssetSource, SessionId } from "@/core/ipc/types/xrf-app";
import { ETextureEncodingFormat } from "@/core/ipc/types/xrf-texture";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const texturesBulkRoutes = {
  readCandidate: (sessionId: SessionId, format: ETextureEncodingFormat): IBulkCall => ({
    args: { sessionId, format },
    route: "textures/read_candidate",
  }),
  readTexels: (roots: XrayRoots, source: AssetSource): IBulkCall => ({
    args: { roots, source },
    route: "textures/read_texels",
  }),
};
