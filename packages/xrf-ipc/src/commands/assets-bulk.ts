// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "#/bulk/bulk-call";
import { IIpcBulkRouteSignature } from "#/signature";
import { AssetSource } from "#/types/xrf-app-core";
import { XrayRoots } from "#/types/xrf-vfs";

/** Every bulk route of the `assets` plugin, as the transport serves it. */
export enum EAssetsBulkRoute {
  READ_ASSET = "assets/read_asset",
  READ_TEXTURE = "assets/read_texture",
}

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const assetsBulkRoutes = {
  readAsset: (roots: XrayRoots, logicalPath: string): IBulkCall => ({
    args: { roots, logicalPath },
    route: EAssetsBulkRoute.READ_ASSET,
  }),
  readTexture: (roots: XrayRoots, source: AssetSource): IBulkCall => ({
    args: { roots, source },
    route: EAssetsBulkRoute.READ_TEXTURE,
  }),
};

/** What each bulk route of the `assets` plugin takes, keyed by its route. */
export interface IAssetsBulkRouteSignatures {
  [EAssetsBulkRoute.READ_ASSET]: IIpcBulkRouteSignature<typeof assetsBulkRoutes.readAsset, ["roots", "logicalPath"]>;
  [EAssetsBulkRoute.READ_TEXTURE]: IIpcBulkRouteSignature<typeof assetsBulkRoutes.readTexture, ["roots", "source"]>;
}
