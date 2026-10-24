// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { SessionId } from "@/core/ipc/types/xrf-app";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const spriteEquipmentBulkRoutes = {
  readPreview: (sessionId: SessionId): IBulkCall => ({ args: { sessionId }, route: "sprite-equipment/read_preview" }),
};
