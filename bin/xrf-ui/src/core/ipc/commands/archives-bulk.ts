// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { IIpcBulkRouteSignature } from "@/core/ipc/signature";
import { SessionId } from "@/core/ipc/types/xrf-app-core";

/** Every bulk route of the `archives` plugin, as the transport serves it. */
export enum EArchivesBulkRoute {
  READ_ENTRY = "archives/read_entry",
  READ_TEXTURE = "archives/read_texture",
}

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const archivesBulkRoutes = {
  readEntry: (sessionId: SessionId, name: string): IBulkCall => ({
    args: { sessionId, name },
    route: EArchivesBulkRoute.READ_ENTRY,
  }),
  readTexture: (sessionId: SessionId, name: string): IBulkCall => ({
    args: { sessionId, name },
    route: EArchivesBulkRoute.READ_TEXTURE,
  }),
};

/** What each bulk route of the `archives` plugin takes, keyed by its route. */
export interface IArchivesBulkRouteSignatures {
  [EArchivesBulkRoute.READ_ENTRY]: IIpcBulkRouteSignature<typeof archivesBulkRoutes.readEntry, ["sessionId", "name"]>;
  [EArchivesBulkRoute.READ_TEXTURE]: IIpcBulkRouteSignature<
    typeof archivesBulkRoutes.readTexture,
    ["sessionId", "name"]
  >;
}
