import { SessionId } from "@/core/bindings/types/xrf-app";
import { ISessionIdentity } from "@/core/ipc/session/session.types";
import { Nullable } from "@/lib/types/general";

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
