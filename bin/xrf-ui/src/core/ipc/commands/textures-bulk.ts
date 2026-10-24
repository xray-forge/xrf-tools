// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { IIpcBulkRouteSignature } from "@/core/ipc/signature";
import { AssetSource, SessionId } from "@/core/ipc/types/xrf-app-core";
import { ETextureEncodingFormat } from "@/core/ipc/types/xrf-texture";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Every bulk route of the `textures` plugin, as the transport serves it. */
export enum ETexturesBulkRoute {
  READ_CANDIDATE = "textures/read_candidate",
  READ_TEXELS = "textures/read_texels",
}

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const texturesBulkRoutes = {
  readCandidate: (sessionId: SessionId, format: ETextureEncodingFormat): IBulkCall => ({
    args: { sessionId, format },
    route: ETexturesBulkRoute.READ_CANDIDATE,
  }),
  readTexels: (roots: XrayRoots, source: AssetSource): IBulkCall => ({
    args: { roots, source },
    route: ETexturesBulkRoute.READ_TEXELS,
  }),
};

/** What each bulk route of the `textures` plugin takes, keyed by its route. */
export interface ITexturesBulkRouteSignatures {
  [ETexturesBulkRoute.READ_CANDIDATE]: IIpcBulkRouteSignature<
    typeof texturesBulkRoutes.readCandidate,
    ["sessionId", "format"]
  >;
  [ETexturesBulkRoute.READ_TEXELS]: IIpcBulkRouteSignature<typeof texturesBulkRoutes.readTexels, ["roots", "source"]>;
}
