import { IDocumentSession, IDocumentSnapshot, TDocument } from "@/core/ipc/document/document.types";
import { Nullable } from "@/lib/types/general";

/** Projects the envelope into view data, retaining its identity and nested references. */
export function restoreDocument<T>(snapshot: IDocumentSnapshot<T>): TDocument<T> {
  return { ...snapshot.document, sessionId: snapshot.sessionId };
}

/** Refuses reads when the owning view has no document. */
export function requireDocumentSession(document: Nullable<IDocumentSession>): string {
  if (!document) {
    throw new Error("No document is open");
  }

  return document.sessionId;
}
