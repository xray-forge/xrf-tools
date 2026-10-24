// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "#/bulk/bulk-call";
import { IIpcBulkRouteSignature } from "#/signature";
import { AssetSource, SessionId } from "#/types/xrf-app-core";
import { ETextureEncodingFormat } from "#/types/xrf-texture";
import { XrayRoots } from "#/types/xrf-vfs";

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
