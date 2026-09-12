import { SessionId } from "@/core/bindings/types/xrf-app";

/** Identifies the native opening that owns a value. */
export interface ISessionIdentity {
  sessionId: SessionId;
}
