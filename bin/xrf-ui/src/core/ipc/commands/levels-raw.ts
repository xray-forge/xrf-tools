// Auto-generated rust bindings. Do not edit it manually.

import { invokeRaw } from "@/core/ipc/raw";
import { SessionId } from "@/core/ipc/types/xrf-app";

/** Commands answering with raw bytes, which Specta cannot type. */
export const levelsRawCommands = {
  readSector: (sessionId: SessionId, sectorId: SessionId): Promise<ArrayBuffer> =>
    invokeRaw("plugin:levels|read_sector", { sessionId, sectorId }),
};
