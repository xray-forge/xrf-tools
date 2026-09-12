import { SessionSnapshot } from "@/core/bindings/types/xrf-app";
import { InvokeHandler } from "@/fixtures/mocks/tauri.mocks";
import { Optional } from "@/lib/types/general";

/** A native snapshot with an explicit identity for restoration and state fixtures. */
export function mockSessionSnapshot<T>(value: T, sessionId: string = "fixture-session"): SessionSnapshot<T> {
  return { sessionId, value };
}

/** Makes a configured open answer echo the caller's identity, including deferred responses. */
export function mockSessionResponse(response: unknown | InvokeHandler): InvokeHandler {
  return async (args) => {
    const value: unknown = typeof response === "function" ? await response(args) : response;

    if (value === null || value === undefined) {
      return null;
    }

    const request = args?.request as Optional<Record<string, unknown>>;
    const id: unknown = args?.openingId ?? args?.motionId ?? args?.sessionId ?? request?.sessionId;

    return mockSessionSnapshot(value, typeof id === "string" ? id : "fixture-session");
  };
}
