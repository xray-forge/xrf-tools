import { Nullable } from "@xrf/types";

import { SessionId } from "@/core/ipc/types/xrf-app-core";

import { ISessionIdentity, ISessionRevision } from "./session.types";

/**
 * Requires an opening's identity before a session-addressed command can run.
 *
 * @param session - Identity held by the owning view.
 * @returns Its native session identity.
 *
 * @throws {Error} When no session is open.
 */
export function requireSessionId(session: Nullable<ISessionIdentity>): SessionId {
  if (!session) {
    throw new Error("No session is open");
  }

  return session.sessionId;
}

/**
 * @param held - What a view shows of a session, as read at its revision.
 * @param moved - Where a change left some session, such as a `SessionChange`.
 * @returns Whether that session moved past what the view read, so the view reads it again.
 */
export function isSessionMovedPast(held: ISessionRevision, moved: ISessionRevision): boolean {
  return moved.sessionId === held.sessionId && moved.revision > held.revision;
}

/**
 * @param held - What a view shows of a session.
 * @param answer - A later answer about the same or another session.
 * @returns Whether the answer is of the same session but read at an older revision, having lost a race to a newer
 *   read; the view drops it.
 */
export function isOlderSessionAnswer(held: ISessionRevision, answer: ISessionRevision): boolean {
  return answer.sessionId === held.sessionId && answer.revision < held.revision;
}
