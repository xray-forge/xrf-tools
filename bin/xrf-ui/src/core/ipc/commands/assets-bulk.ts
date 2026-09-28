// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const assetsBulkRoutes = {
  readAsset: (roots: XrayRoots, logicalPath: string): IBulkCall => ({
    args: { roots, logicalPath },
    route: "assets/read_asset",
  }),
};
