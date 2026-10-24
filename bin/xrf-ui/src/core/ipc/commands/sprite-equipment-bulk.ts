// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { IIpcBulkRouteSignature } from "@/core/ipc/signature";
import { SessionId } from "@/core/ipc/types/xrf-app-core";

/** Every bulk route of the `sprite-equipment` plugin, as the transport serves it. */
export enum ESpriteEquipmentBulkRoute {
  READ_PREVIEW = "sprite-equipment/read_preview",
}

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const spriteEquipmentBulkRoutes = {
  readPreview: (sessionId: SessionId): IBulkCall => ({
    args: { sessionId },
    route: ESpriteEquipmentBulkRoute.READ_PREVIEW,
  }),
};

/** What each bulk route of the `sprite-equipment` plugin takes, keyed by its route. */
export interface ISpriteEquipmentBulkRouteSignatures {
  [ESpriteEquipmentBulkRoute.READ_PREVIEW]: IIpcBulkRouteSignature<
    typeof spriteEquipmentBulkRoutes.readPreview,
    ["sessionId"]
  >;
}
