import { DocumentSnapshot } from "@/core/bindings/types/xrf-app";
import { InvokeHandler } from "@/fixtures/mocks/tauri.mocks";
import { Optional } from "@/lib/types/general";

/** A native snapshot with an explicit identity for restoration and state fixtures. */
export function mockDocument<T>(document: T, sessionId: string = "fixture-session"): DocumentSnapshot<T> {
  return { sessionId, document };
}

/** Makes a configured open answer echo the caller's identity, including deferred responses. */
export function mockDocumentResponse(response: unknown | InvokeHandler): InvokeHandler {
  return async (args) => {
    const document: unknown = typeof response === "function" ? await response(args) : response;

    if (document === null || document === undefined) {
      return null;
    }

    const request = args?.request as Optional<Record<string, unknown>>;
    const id: unknown = args?.openingId ?? args?.motionId ?? args?.sessionId ?? request?.sessionId;

    return mockDocument(document, typeof id === "string" ? id : "fixture-session");
  };
}
