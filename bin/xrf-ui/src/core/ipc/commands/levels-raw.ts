// Auto-generated rust bindings. Do not edit it manually.

import { invokeRaw } from "@/core/ipc/raw";
import { SessionId } from "@/core/ipc/types/xrf-app";

/** Commands answering with raw bytes, which Specta cannot type. */
export const levelsRawCommands = {
  readDetails: (sessionId: SessionId, detailsId: SessionId): Promise<ArrayBuffer> =>
    invokeRaw("plugin:levels|read_details", { sessionId, detailsId }),
  readSector: (sessionId: SessionId, sectorId: SessionId): Promise<ArrayBuffer> =>
    invokeRaw("plugin:levels|read_sector", { sessionId, sectorId }),
  readSpawnModel: (sessionId: SessionId, name: string): Promise<ArrayBuffer> =>
    invokeRaw("plugin:levels|read_spawn_model", { sessionId, name }),
};
