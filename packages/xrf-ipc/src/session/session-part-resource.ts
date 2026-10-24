import { EventBus, EventUnsubscribe, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { assertExhaustive, Maybe, Nullable } from "@xrf/types";

import { SESSION_CHANGE_EVENT } from "#/session/session.types";
import { isOlderSessionAnswer, isSessionMovedPast } from "#/session/session.utils";
import { ESessionRootChange, SessionChange, SessionId, SessionSnapshot } from "#/types/xrf-app-core";

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
 * Attached to an opening, it reads its part and reads it again once a change names one of the parts it is read from
 * stale or derives the opening anew, past what it holds; an answer older than the one held, or for an opening it no
 * longer follows, is dropped. A change it cannot read past yet (`deriving`, `stale`, `failed`) leaves what it holds as
 * it is: whoever holds the opening's root says it is stale (`LevelLoadService.isStale` for a level). A failed read
 * keeps what was shown.
 */
export class SessionPartResource<T> {
  /** The part's last answer, with its opening and revision. */
  @Observable()
  private answer: AsyncState<SessionSnapshot<T>> = AsyncState.idle<SessionSnapshot<T>>();

  /** The opening followed, or none. */
  private sessionId: Nullable<SessionId> = null;

  /** Reads begun, so one overtaken by a later read, or by another opening, is dropped. */
  private reads: number = 0;

  /** The latest read begun, which an attach waits for through any that overtake it. */
  private pending: Promise<void> = Promise.resolve();

  private unsubscribe: Nullable<EventUnsubscribe> = null;

  /**
   * @param eventBus - The bus the session changes are told on.
   * @param parts - The names of the parts it is read from, generated `*_PART` constants: what a change names stale.
   * @param query - The part's `describe_<part>` query.
   */
  public constructor(
    private readonly eventBus: EventBus,
    private readonly parts: ReadonlyArray<string>,
    private readonly query: TSessionPartQuery<T>
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /** The part's read as a view shows its lifecycle: what is held kept while a read goes on, or where one fails. */
  @Computed()
  public get state(): AsyncState<T> {
    return this.answer.map((snapshot: SessionSnapshot<T>) => snapshot.value);
  }

  /** The part as last read, or none before it is. */
  @Computed()
  public get value(): Nullable<T> {
    return this.answer.value?.value ?? null;
  }

  /** The part's snapshot as last read: the value, its opening and the revision it was read at. */
  @Computed()
  public get snapshot(): Nullable<SessionSnapshot<T>> {
    return this.answer.value;
  }

  @Computed()
  public get isLoading(): boolean {
    return this.answer.isLoading;
  }

  /** Why the last read failed, or none. */
  @Computed()
  public get error(): Nullable<Error> {
    return this.answer.error;
  }

  /**
   * Follows `sessionId`'s part, reading it now; another opening followed before is let go.
   *
   * @param sessionId - The opening whose part to read.
   * @returns Resolves once the latest read settles: a change overtaking this one is waited for too, so the part is
   *   read, or has failed, by then.
   */
  @BoundAction()
  public attach(sessionId: SessionId): Promise<void> {
    if (this.sessionId !== sessionId) {
      this.detach();
      this.sessionId = sessionId;
      this.unsubscribe = this.eventBus.subscribe(SESSION_CHANGE_EVENT, this.onSessionChange);
    }

    void this.fetch();

    return this.settle();
  }

  /** Waits for the latest read, and for each that overtakes it meanwhile. */
  private async settle(): Promise<void> {
    let pending: Promise<void>;

    do {
      pending = this.pending;
      await pending;
    } while (pending !== this.pending);
  }

  /** Lets the opening go, and what was read of it. */
  @BoundAction()
  public detach(): void {
    this.unsubscribe?.();
    this.unsubscribe = null;
    this.sessionId = null;
    this.reads += 1;
    this.answer = this.answer.asIdle(null);
  }

  /**
   * Reads the part again where `change` names it stale or derived its opening anew past what is held.
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
          void this.fetch();
        }

        return;

      case ESessionRootChange.DERIVING:
      case ESessionRootChange.STALE:
      case ESessionRootChange.FAILED:
        return;

      default:
        return assertExhaustive(change.root);
    }
  }

  /** Reads the part of the opening followed, keeping what was shown until the answer lands. */
  private fetch(): Promise<void> {
    const sessionId: Nullable<SessionId> = this.sessionId;
    const read: number = ++this.reads;

    if (!sessionId) {
      return this.pending;
    }

    this.noteLoading();
    // An answer that cannot be taken fails the read as a query that failed does.
    this.pending = this.query(sessionId)
      .then((answer: SessionSnapshot<T>) => this.noteAnswer(read, answer))
      .catch((error: unknown) => this.noteFailure(read, transformError(error)));

    return this.pending;
  }

  @BoundAction()
  private noteLoading(): void {
    this.answer = this.answer.asLoading();
  }

  @BoundAction()
  private noteAnswer(read: number, answer: SessionSnapshot<T>): void {
    const held: Nullable<SessionSnapshot<T>> = this.answer.value;

    if (read !== this.reads || answer.sessionId !== this.sessionId || (held && isOlderSessionAnswer(held, answer))) {
      return;
    }

    this.answer = this.answer.asReady(answer);
  }

  @BoundAction()
  private noteFailure(read: number, error: Error): void {
    if (read === this.reads) {
      this.answer = this.answer.asFailed(error, this.answer.value);
    }
  }

  /** Whether `change` names a part it is read from stale, or derived the opening anew, which lets every part go. */
  private isNamedBy(change: SessionChange): boolean {
    return (
      change.root.kind === ESessionRootChange.DERIVED ||
      change.staleParts.some((part: string) => this.parts.includes(part))
    );
  }

  /** Whether `change` moved the opening past the revision the part was read at; anything does before it is read. */
  private isMovedPast(change: SessionChange): boolean {
    const held: Nullable<SessionSnapshot<T>> = this.answer.value;

    return !held || isSessionMovedPast(held, change);
  }
}
