import { ISessionIdentity, Session } from "@xrf/ipc/session";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { Nullable, Optional } from "@xrf/types";

/** The arguments a session-opening command was invoked with, as far as a fixture reads them for an identity. */
export type MockSessionArgs = Record<string, unknown>;

/** What a fixture opens: the value, or a reader of it given the arguments the open was invoked with. */
export type MockSessionValue<T, A extends MockSessionArgs = MockSessionArgs> = T | ((args: A) => T | Promise<T>);

/** A native snapshot with an explicit identity for restoration and state fixtures, at the revision an open leaves. */
export function mockSessionSnapshot<T>(
  value: T,
  sessionId: string = "fixture-session",
  revision: number = 0
): SessionSnapshot<T> {
  return { sessionId, revision, value };
}

/**
 * Hands a service the opening a fixture installed straight into its state.
 *
 * @param service - Service holding the session to seed.
 * @param session - Identity the fixture put on screen.
 * @returns The same identity, so it can be passed straight into the state it seeds.
 */
export function mockRestoredSession<T extends ISessionIdentity>(service: object, session: T): T {
  (service as unknown as { session: Session }).session.adopt(session);

  return session;
}

/**
 * What a fixture's open answers: a snapshot of what it opens, or none where it opens nothing, as a restore holding
 * nothing answers.
 */
export type MockSessionAnswer<T> = null extends T ? Nullable<SessionSnapshot<NonNullable<T>>> : SessionSnapshot<T>;

/** Makes a configured open answer echo the caller's identity, including deferred responses. */
export function mockSessionResponse<T, A extends MockSessionArgs = MockSessionArgs>(
  response: MockSessionValue<T, A>
): (args: A) => Promise<MockSessionAnswer<T>> {
  return async (args: A): Promise<MockSessionAnswer<T>> => {
    const value: T = typeof response === "function" ? await (response as (args: A) => T | Promise<T>)(args) : response;

    if (value === null || value === undefined) {
      return null as MockSessionAnswer<T>;
    }

    const request = args?.request as Optional<Record<string, unknown>>;
    const id: unknown = args?.motionId ?? args?.sectorId ?? args?.sessionId ?? request?.sessionId;

    return mockSessionSnapshot(value, typeof id === "string" ? id : "fixture-session") as MockSessionAnswer<T>;
  };
}
