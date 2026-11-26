// Auto-generated rust bindings. Do not edit it manually.

import { XrayRevision } from "#/types/xrf-vfs";

/** A single opening, allocated by its caller before dispatch so it can also be closed while pending. */
export type SessionId = string;

/** An immutable value addressed by the opening that produced it; a held snapshot survives close. */
export type SessionSnapshot<T> = {
  sessionId: SessionId;
  /** The project revision the snapshot is current at: what a change moved it to, or where an open left it. */
  revision: XrayRevision;
  /**
   * Shared with the snapshots a change moves the session to while what it was read from stays current, so a commit
   * that touched none of it copies nothing.
   */
  value: T;
};
