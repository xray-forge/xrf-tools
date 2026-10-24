import { EventType } from "@wirestate/core";

import { SessionId } from "#/types/xrf-app-core";
import { XrayRevision } from "#/types/xrf-vfs";

/** Identifies the native opening that owns a value. */
export interface ISessionIdentity {
  sessionId: SessionId;
}

/** Identifies an opening and the project revision a value was read from it at, as every snapshot does. */
export interface ISessionRevision extends ISessionIdentity {
  revision: XrayRevision;
}

/** What a scope's release retires: the openings one service holds in it, which the backend closes with the scope. */
export interface IScopedSession {
  /** Forgets every opening without closing it one by one, so a publication landing later closes itself. */
  retire(): void;
}

/**
 * What one committed change did to one open session, a `SessionChange`, told to every service: the one holding that
 * session's snapshot reads again what went stale.
 */
export const SESSION_CHANGE_EVENT: EventType = Symbol("@/ipc/session/change");
