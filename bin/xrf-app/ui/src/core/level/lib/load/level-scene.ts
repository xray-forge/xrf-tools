import { SessionId } from "@xrf/ipc/types/xrf-app-core";
import { XrayRevision } from "@xrf/ipc/types/xrf-vfs";

/** The scene an open level streams as: its session, and the revision it last streamed afresh at. */
export interface ILevelScene {
  sessionId: SessionId;
  revision: XrayRevision;
}
