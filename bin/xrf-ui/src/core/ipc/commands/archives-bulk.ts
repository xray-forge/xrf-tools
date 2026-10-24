// Auto-generated rust bindings. Do not edit it manually.

import { IBulkCall } from "@/core/ipc/bulk";
import { SessionId } from "@/core/ipc/types/xrf-app";

/** Routes answering with bytes over the loopback transport, which Specta never sees. */
export const archivesBulkRoutes = {
  readEntry: (sessionId: SessionId, name: string): IBulkCall => ({
    args: { sessionId, name },
    route: "archives/read_entry",
  }),
  readTexture: (sessionId: SessionId, name: string): IBulkCall => ({
    args: { sessionId, name },
    route: "archives/read_texture",
  }),
};
