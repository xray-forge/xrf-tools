import { EventBus, EventUnsubscribe, inject, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { cancelFlows, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import {
  IScopedSession,
  isOlderSessionAnswer,
  isSessionMovedPast,
  Session,
  SESSION_CHANGE_EVENT,
  SessionScopeService,
} from "@xrf/ipc/session";
import {
  ESessionRootChange,
  SessionChange,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";
import { assertExhaustive, Maybe, Nullable } from "@xrf/types";

import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandReporter, TCommandCompletion } from "@/core/operations/lib/command-reporter";

/** One open of a root: what it opens, as log lines and a failure name it, and the command taking the new identity. */
export interface ISessionRootOpening<T> {
  label: string;
  command: (sessionId: SessionId) => Promise<SessionSnapshot<T>>;
}

/** How a root follows the commits the backend tells of the opening held. */
export interface ISessionRootFollow<T> {
  /** Reads the held opening again once a change derives it anew: the domain's `restore` command. */
  reread: (tool: string) => Promise<SessionRestore<T>>;
  /** The title of the warning a change the backend could not read past raises, or none where a view shows it. */
  staleNotice?: string;
  /** Hears every change to the opening held, a change heard while it opened included, for what the owner follows. */
  heard?: (change: SessionChange) => void;
  /** Hears the held root read again past a change, for the owner's reads that describe it as it was. */
  refreshed?: (snapshot: SessionSnapshot<T>) => void;
}

/** What a tool holds open and how it is let go and found again. */
export interface ISessionRootOptions<T> {
  /** What the root is, as log lines and notices name it: `exports project`. */
  subject: string;
  /** Releases openings: the domain's generated `close` command. */
  close: (sessionIds: Array<SessionId>) => Promise<unknown>;
  /** Reads what the tool still holds after a reload: the domain's generated `restore` command; none where nothing is. */
  restore?: (tool: string) => Promise<SessionRestore<T>>;
  /** Follows the backend's commits to the held opening, for a root an edit can change. */
  follow?: ISessionRootFollow<T>;
  /**
   * Whether a failure lets the held root go rather than keep it, for a view that must never show a root it was asked
   * to leave: an open that fails closes the root it was to replace, and a close that fails still forgets it, the
   * opening staying owned for the next close to release.
   */
  isLetGoOnFailure?: boolean;
  /**
   * Reads what must be on screen with the root before it shows, inside the flow that opened it: where a level starts,
   * so the viewer meets it whole.
   */
  adopting?: (snapshot: SessionSnapshot<T>) => TFlow;
  /**
   * Reads what the root's view needs beside it, inside the flow that adopted the root, so whatever supersedes that flow
   * cancels the read too: the listing of what a root holds. Returns nothing where all it does is let the last root's
   * reads go.
   */
  adopted?: (snapshot: SessionSnapshot<T>) => TFlow | void;
}

/**
 * The root a tool holds open, as the page shows it: one opening at a time, its snapshot, and how it opens, refreshes,
 * closes and is restored. The sibling of `SessionPartResource`, which reads the root's parts.
 */
export class SessionRootResource<T> implements IScopedSession {
  /**
   * Whether the backend has been asked what the tool still holds, a restore cancelled or failed included, or a root
   * opened since, which a restore asked for meanwhile joins rather than runs. Set as a root is adopted, before the reads
   * beside it: a root is on screen without its listing finished.
   */
  @Observable()
  public isRestored: boolean = false;

  /** Whether a change moved what the held root was read from: it shows as it was read until it is read again. */
  @Observable()
  public isStale: boolean = false;

  @Observable()
  private root: AsyncState<SessionSnapshot<T>> = AsyncState.idle<SessionSnapshot<T>>();

  private readonly session: Session;

  /**
   * The changes heard while an open or a restore is under way, for any opening but the one held: the new opening's id
   * is known only once its answer resumes the flow, so a change to it may come first. Those of the opening adopted, no
   * older than it, are heard then.
   */
  private heard: Nullable<Array<SessionChange>> = null;

  private unsubscribe: Nullable<EventUnsubscribe> = null;

  /**
   * @param log - The owning service's logger.
   * @param options - What is held and how it is let go and found again.
   * @param scopeService - The scope of the tool holding the root.
   * @param eventBus - The bus the backend's changes and the notices are told on.
   * @param reporter - Reports each verb; built from the container's bus and tool by default.
   */
  public constructor(
    private readonly log: Logger,
    private readonly options: ISessionRootOptions<T>,
    scopeService: SessionScopeService = inject(SessionScopeService),
    eventBus: EventBus = inject(EventBus),
    private readonly reporter: CommandReporter = new CommandReporter(log, eventBus)
  ) {
    this.session = new Session(options.close, scopeService);
    scopeService.register(this);

    // Composed state is not activated by the container's observable plugin; it binds the handler subscribed below.
    makeObservable(this);

    if (options.follow) {
      this.unsubscribe = eventBus.subscribe(SESSION_CHANGE_EVENT, this.onSessionChange);
    }
  }

  /** The root's lifecycle and value, the opening it came from left out. */
  @Computed()
  public get state(): AsyncState<T> {
    return this.root.map((snapshot: SessionSnapshot<T>) => snapshot.value);
  }

  /** The root as last read, with its opening and revision, or none. */
  @Computed()
  public get snapshot(): Nullable<SessionSnapshot<T>> {
    return this.root.value;
  }

  /** The root as last read, or none while nothing is open. */
  @Computed()
  public get value(): Nullable<T> {
    return this.root.value?.value ?? null;
  }

  /** The identity every read of the root is addressed by, or none while nothing is open. */
  @Computed()
  public get sessionId(): Nullable<SessionId> {
    return this.root.value?.sessionId ?? null;
  }

  @Computed()
  public get isLoading(): boolean {
    return this.root.isLoading;
  }

  /** Why the last open, refresh or restore failed, or none. */
  @Computed()
  public get error(): Nullable<Error> {
    return this.root.error;
  }

  /**
   * Takes back what the tool still holds, unless a person's request has taken the lane or a root opened since is held.
   */
  @ExclusiveFlow("state")
  public *restore(): TFlow {
    const { restore } = this.options;

    try {
      if (this.root.value || !restore) {
        return;
      }

      this.heard = [];

      const completion: TCommandCompletion<SessionRestore<T>> = yield* this.readRestore(restore);

      // A tool that holds nothing leaves the root as it is: `isRestored` says the backend was asked.
      if (completion.error) {
        this.root = this.root.asFailed(completion.error);
      } else if (completion.result) {
        this.session.adopt(completion.result);

        yield* this.adoptRoot(completion.result);
      }
    } finally {
      this.heard = null;
      // Reached cancelled too: a tool left waiting for its restore shows a spinner for the rest of the session.
      this.isRestored = true;
    }
  }

  /**
   * Opens a root in place of the one held, which a failure leaves on screen.
   *
   * @param label - What is opened, as the log lines and a failure name it: a path, a root.
   * @param command - The domain's open command, taking the new identity first, or an adapter for a request object.
   * @param args - The command's remaining arguments.
   * @returns Whether the root opened, which a picker waits on before it gives way to what opened.
   */
  @LatestFlow("state")
  public *open<Args extends Array<unknown>>(
    label: string,
    command: (sessionId: SessionId, ...args: Args) => Promise<SessionSnapshot<T>>,
    ...args: Args
  ): TFlow<boolean> {
    return yield* this.replace(
      `Open ${this.options.subject}`,
      `Could not open the ${this.options.subject}`,
      label,
      () => this.session.open(command, ...args)
    );
  }

  /**
   * Opens the held root again, joining an open or a refresh already under way; nothing while nothing is open.
   *
   * @param label - What is opened again, as the log lines and a failure name it.
   * @param command - The domain's open command, as {@link open} takes it.
   * @param args - The command's remaining arguments.
   * @returns Whether the root opened again.
   */
  @ExclusiveFlow("state")
  public *refresh<Args extends Array<unknown>>(
    label: string,
    command: (sessionId: SessionId, ...args: Args) => Promise<SessionSnapshot<T>>,
    ...args: Args
  ): TFlow<boolean> {
    if (!this.root.value) {
      return false;
    }

    return yield* this.replace(
      `Refresh ${this.options.subject}`,
      `Could not refresh the ${this.options.subject}`,
      label,
      () => this.session.open(command, ...args)
    );
  }

  /**
   * Lets the root go; a failure keeps it on screen, as it is still open.
   *
   * @returns Whether the root was let go.
   */
  @LatestFlow("state")
  public *close(): TFlow<boolean> {
    const held: AsyncState<SessionSnapshot<T>> = this.root;

    this.root = this.root.asLoading();

    const completion: TCommandCompletion<void> = yield* this.letGo();

    if (completion.error === null || this.options.isLetGoOnFailure) {
      this.root = this.root.asIdle();
      this.isStale = false;
    } else {
      // A close that overtook an open keeps what that open kept on screen, which nothing is loading any more.
      this.root = held.isLoading ? held.asReady() : held;
    }

    return completion.error === null;
  }

  /**
   * Shows a newer answer of the held opening: what a save left, or a read after a change. One older than what is held,
   * of another opening, or arriving while an open or a close decides what is shown is dropped.
   *
   * @param answer - The opening as the backend now holds it.
   * @returns Whether it is shown.
   */
  @BoundAction()
  public accept(answer: SessionSnapshot<T>): boolean {
    const held: Nullable<SessionSnapshot<T>> = this.root.value;

    if (!held || this.root.isLoading || answer.sessionId !== held.sessionId || isOlderSessionAnswer(held, answer)) {
      return false;
    }

    this.root = this.root.asReady(answer);
    this.isStale = false;

    return true;
  }

  /**
   * Projects the held snapshot, its opening included, keeping the root's lifecycle and failure.
   *
   * @param project - Projection applied only while a root is held.
   * @returns The projected state.
   */
  public mapSnapshot<U>(project: (snapshot: SessionSnapshot<T>) => U): AsyncState<U> {
    return this.root.map(project);
  }

  /**
   * Lets the root go at once, for a teardown that cannot wait for a close: cancels what runs, forgets what is held, and
   * closes the openings in the background, while the tool stays.
   */
  @BoundAction()
  public release(): void {
    cancelFlows(this);
    this.session.release();
    this.forget();
  }

  /** Cancels what runs and forgets what is held as the tool is left; the scope's release closes the openings. */
  @BoundAction()
  public retire(): void {
    cancelFlows(this);
    this.unsubscribe?.();
    this.unsubscribe = null;
    this.forget();
  }

  private forget(): void {
    this.root = this.root.asIdle();
    this.isStale = false;
    this.heard = null;
  }

  private *replace(
    name: string,
    failure: string,
    label: string,
    command: () => Promise<SessionSnapshot<T>>
  ): TFlow<boolean> {
    this.root = this.root.asLoading();
    this.heard = this.options.follow ? [] : null;

    try {
      const completion: TCommandCompletion<SessionSnapshot<T>> = yield* this.reporter.run({
        name,
        subject: label,
        command,
        failure,
      });

      if (completion.error) {
        if (this.options.isLetGoOnFailure && this.root.value) {
          yield* this.letGo();
        }

        this.root = this.root.asFailed(completion.error, this.options.isLetGoOnFailure ? null : this.root.value);

        return false;
      }

      yield* this.adoptRoot(completion.result);

      return true;
    } finally {
      this.heard = null;
      this.isRestored = true;
    }
  }

  /** Closes the openings held, a failure leaving them owned for the next close. */
  private *letGo(): TFlow<TCommandCompletion<void>> {
    return yield* this.reporter.run({
      name: `Close ${this.options.subject}`,
      command: () => this.session.close(),
      failure: `Could not close the ${this.options.subject}`,
    });
  }

  /**
   * Takes an opening as the root held, whether opened or restored: the changes heard for it while it opened are heard
   * now, then the owner's reads beside it run in the adopting flow.
   */
  private *adoptRoot(snapshot: SessionSnapshot<T>): TFlow {
    if (this.options.adopting) {
      yield* this.options.adopting(snapshot);
    }

    const heard: Array<SessionChange> = (this.heard ?? []).filter(
      (change: SessionChange) => change.sessionId === snapshot.sessionId && !isOlderSessionAnswer(snapshot, change)
    );

    this.heard = null;
    this.root = this.root.asReady(snapshot);
    this.isStale = false;
    this.isRestored = true;

    for (const change of heard) {
      this.followChange(snapshot, change);
    }

    const reads: TFlow | void = this.options.adopted?.(snapshot);

    if (reads) {
      yield* reads;
    }
  }

  /** Reads what the tool still holds, a failure reported as a warning. */
  private *readRestore<A>(restore: (tool: string) => Promise<A>): TFlow<TCommandCompletion<A>> {
    return yield* this.reporter.run({
      name: `Restore ${this.options.subject}`,
      command: () => restore(this.session.tool),
      failure: `Could not restore the open ${this.options.subject}`,
      failureSeverity: ENotificationSeverity.WARNING,
    });
  }

  /**
   * Follows a change to the opening held, and keeps one to any other while an open or a restore may yet adopt it.
   *
   * @param event - A change to some open session.
   */
  @BoundAction()
  private onSessionChange(event: WireEvent<SessionChange>): void {
    const change: Maybe<SessionChange> = event.payload;
    const held: Nullable<SessionSnapshot<T>> = this.root.value;

    if (!change) {
      return;
    }

    if (held && held.sessionId === change.sessionId) {
      this.followChange(held, change);
    } else {
      this.heard?.push(change);
    }
  }

  /**
   * Reads the held root again once its opening derived it anew past what is held; a root whose own files moved is
   * stale meanwhile, and stays as it was read where the backend could not read past the change, with a warning.
   */
  @BoundAction()
  private followChange(held: SessionSnapshot<T>, change: SessionChange): void {
    this.options.follow?.heard?.(change);

    switch (change.root.kind) {
      case ESessionRootChange.KEPT:
        return;

      case ESessionRootChange.DERIVED:
        if (isSessionMovedPast(held, change)) {
          this.isStale = true;
          void this.reread();
        }

        return;

      case ESessionRootChange.DERIVING:
        this.isStale = true;

        return;

      case ESessionRootChange.STALE:
      case ESessionRootChange.FAILED:
        this.isStale = true;
        this.warnStale(change);

        return;

      default:
        return assertExhaustive(change.root);
    }
  }

  private warnStale(change: SessionChange): void {
    const title: Maybe<string> = this.options.follow?.staleNotice;

    this.log.warn(`The ${this.options.subject} was not read again:`, change.root);

    if (title) {
      this.reporter.notify(ENotificationSeverity.WARNING, {
        details: change.root.kind === ESessionRootChange.FAILED ? change.root.reason : undefined,
        title,
      });
    }
  }

  /** Reads the held opening again after a change; a failure leaves what is shown, stale. */
  private async reread(): Promise<void> {
    const reread: Maybe<(tool: string) => Promise<SessionRestore<T>>> = this.options.follow?.reread;

    if (!reread) {
      return;
    }

    try {
      const answer: SessionRestore<T> = await reread(this.session.tool);

      if (answer && this.accept(answer)) {
        this.log.info(`The ${this.options.subject} was read again:`, `revision ${answer.revision}`);
        this.options.follow?.refreshed?.(answer);
      }
    } catch (error: unknown) {
      this.log.warn(`Could not read the ${this.options.subject} again:`, transformError(error));
    }
  }
}
