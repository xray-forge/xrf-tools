/** A document's native identity, distinct from its path and from any frontend flow. */
export interface IDocumentSession {
  sessionId: string;
}

/** The envelope consumed by document helpers; generated IPC snapshots satisfy this contract. */
export interface IDocumentSnapshot<T> extends IDocumentSession {
  document: T;
}

/** A view's data carries its native identity even after it is restored. */
export type TDocument<T> = T & IDocumentSession;
