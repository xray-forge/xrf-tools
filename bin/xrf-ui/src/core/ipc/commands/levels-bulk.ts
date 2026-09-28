// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { SessionId } from "@/core/ipc/types/xrf-app";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const levelsBulkRoutes = {
  readDetails: (sessionId: SessionId, detailsId: SessionId): IBulkCall => ({
    args: { sessionId, detailsId },
    route: "levels/read_details",
  }),
  readSector: (sessionId: SessionId, sectorId: SessionId): IBulkCall => ({
    args: { sessionId, sectorId },
    route: "levels/read_sector",
  }),
  readSpawnModel: (sessionId: SessionId, name: string): IBulkCall => ({
    args: { sessionId, name },
    route: "levels/read_spawn_model",
  }),
};
