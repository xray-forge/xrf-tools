import { EventType } from "@wirestate/core";

import { SessionId } from "@/core/ipc/types/xrf-app";
import { XrayRevision } from "@/core/ipc/types/xrf-vfs";

/** Identifies the native opening that owns a value. */
export interface ISessionIdentity {
  sessionId: SessionId;
}

/** Identifies an opening and the project revision a value was read from it at, as every snapshot does. */
export interface ISessionRevision extends ISessionIdentity {
  revision: XrayRevision;
}

/**
 * What one committed change did to one open session, a `SessionChange`, told to every service: the one holding that
 * session's snapshot reads again what went stale.
 */
export const SESSION_CHANGE_EVENT: EventType = Symbol("@/ipc/session/change");
