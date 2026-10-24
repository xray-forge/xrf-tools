import { EventBus, EventUnsubscribe, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { assertExhaustive, Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { ESessionRootChange, SessionChange, SessionId, SessionSnapshot } from "@/core/ipc/types/xrf-app";

import { SESSION_CHANGE_EVENT } from "./session.types";
import { isOlderSessionAnswer, isSessionMovedPast } from "./session.utils";

/**
 * Reads one part of an open session: what its `describe_<part>` query answers.
 *
 * @param sessionId - The opening the part belongs to.
 * @returns The part at the revision the backend read it at.
 */
export type TSessionPartQuery<T> = (sessionId: SessionId) => Promise<SessionSnapshot<T>>;

/**
 * One part of an open session as the page shows it, kept with the revision it was read at.
 *
 * Attached to an opening, it reads its part and reads it again once a change names the part stale or derives the
 * opening anew, past what it holds; an answer older than the one held, or for an opening it no longer follows, is
 * dropped. A change it cannot read past yet (`deriving`, `stale`, `failed`) marks what it holds stale meanwhile.
 * A failed read keeps what was shown.
 */
export class SessionPartResource<T> {
  @Observable()
  private state: AsyncState<SessionSnapshot<T>> = AsyncState.idle<SessionSnapshot<T>>();

  /** Whether a change moved what the part was read from, and it shows as read until it is read again. */
  @Observable()
  public isStale: boolean = false;

  /** The opening followed, or none. */
  private sessionId: Nullable<SessionId> = null;

  /** Reads begun, so one overtaken by a later read, or by another opening, is dropped. */
  private reads: number = 0;

  private unsubscribe: Nullable<EventUnsubscribe> = null;

  /**
   * @param eventBus - The bus the session changes are told on.
   * @param part - The part's name, a generated `*_PART` constant: what a change names stale.
   * @param query - The part's `describe_<part>` query.
   */
  public constructor(
    private readonly eventBus: EventBus,
    private readonly part: string,
    private readonly query: TSessionPartQuery<T>
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /** The part as last read, or none before it is. */
  @Computed()
  public get value(): Nullable<T> {
    return this.state.value?.value ?? null;
  }

  /** The part's snapshot as last read: the value, its opening and the revision it was read at. */
  @Computed()
  public get snapshot(): Nullable<SessionSnapshot<T>> {
    return this.state.value;
  }

  @Computed()
  public get isLoading(): boolean {
    return this.state.isLoading;
  }

  /** Why the last read failed, or none. */
  @Computed()
  public get error(): Nullable<Error> {
    return this.state.error;
  }

  /**
   * Follows `sessionId`'s part, reading it now; another opening followed before is let go.
   *
   * @param sessionId - The opening whose part to read.
   * @returns Resolves once the read settles, dropped or not.
   */
  @BoundAction()
  public attach(sessionId: SessionId): Promise<void> {
    if (this.sessionId !== sessionId) {
      this.detach();
      this.sessionId = sessionId;
      this.unsubscribe = this.eventBus.subscribe(SESSION_CHANGE_EVENT, this.onSessionChange);
    }

    return this.read();
  }

  /** Lets the opening go, and what was read of it. */
  @BoundAction()
  public detach(): void {
    this.unsubscribe?.();
    this.unsubscribe = null;
    this.sessionId = null;
    this.reads += 1;
    this.state = this.state.asIdle(null);
    this.isStale = false;
  }

  /**
   * Reads the part again where `change` names it stale or derived its opening anew past what is held, and marks it
   * stale where the opening's own files moved.
   *
   * @param event - A change to some open session.
   */
  @BoundAction()
  private onSessionChange(event: WireEvent<SessionChange>): void {
    const change: Maybe<SessionChange> = event.payload;

    if (!change || change.sessionId !== this.sessionId) {
      return;
    }

    switch (change.root.kind) {
      case ESessionRootChange.KEPT:
      case ESessionRootChange.DERIVED:
        if (this.isNamedBy(change) && this.isMovedPast(change)) {
          this.isStale = true;
          void this.read();
        }

        return;

      case ESessionRootChange.DERIVING:
      case ESessionRootChange.STALE:
      case ESessionRootChange.FAILED:
        this.isStale = true;

        return;

      default:
        return assertExhaustive(change.root);
    }
  }

  /** Reads the part of the opening followed, keeping what was shown until the answer lands. */
  private async read(): Promise<void> {
    const sessionId: Nullable<SessionId> = this.sessionId;
    const read: number = ++this.reads;

    if (!sessionId) {
      return;
    }

    this.noteLoading();

    try {
      this.noteAnswer(read, await this.query(sessionId));
    } catch (error: unknown) {
      this.noteFailure(read, transformError(error));
    }
  }

  @BoundAction()
  private noteLoading(): void {
    this.state = this.state.asLoading();
  }

  @BoundAction()
  private noteAnswer(read: number, answer: SessionSnapshot<T>): void {
    const held: Nullable<SessionSnapshot<T>> = this.state.value;

    if (read !== this.reads || answer.sessionId !== this.sessionId || (held && isOlderSessionAnswer(held, answer))) {
      return;
    }

    this.state = this.state.asReady(answer);
    this.isStale = false;
  }

  @BoundAction()
  private noteFailure(read: number, error: Error): void {
    if (read === this.reads) {
      this.state = this.state.asFailed(error, this.state.value);
    }
  }

  /** Whether `change` names the part stale, or derived the opening anew, which lets every part go. */
  private isNamedBy(change: SessionChange): boolean {
    return change.root.kind === ESessionRootChange.DERIVED || change.staleParts.includes(this.part);
  }

  /** Whether `change` moved the opening past the revision the part was read at; anything does before it is read. */
  private isMovedPast(change: SessionChange): boolean {
    const held: Nullable<SessionSnapshot<T>> = this.state.value;

    return !held || isSessionMovedPast(held, change);
  }
}
