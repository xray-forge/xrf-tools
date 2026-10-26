import { inject } from "@wirestate/core";
import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { cancelFlows, ExclusiveFlow, LatestFlow, TFlow } from "@xrf/flow";
import { IScopedSession, Session, SessionScopeService } from "@xrf/ipc/session";
import { SessionId, SessionRestore, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { ENotificationSeverity } from "@/core/notifications/lib";
import { CommandReporter, TCommandCompletion } from "@/core/operations/lib/command-reporter";

/** What a tool holds open and how it is let go and found again. */
export interface ISessionRootOptions<T> {
  /** What the root is, as log lines and notices name it: `exports project`. */
  subject: string;
  /** Releases openings: the domain's generated `close` command. */
  close: (sessionIds: Array<SessionId>) => Promise<unknown>;
  /** Reads what the tool still holds after a reload: the domain's generated `restore` command; none where nothing is. */
  restore?: (tool: string) => Promise<SessionRestore<T>>;
}

/**
 * The root a tool holds open, as the page shows it: one opening at a time, its snapshot, and how it opens, refreshes,
 * closes and is restored. The sibling of `SessionPartResource`, which reads the root's parts.
 *
 * Every verb runs on one lane. Open and close take it over, so a newer request replaces an older one; a refresh joins
 * whatever runs, as it asks for what is held again; a restore joins too, so a reload's read cannot publish over what
 * the person opened meanwhile. A replacement keeps the
 * root it replaces on screen until it lands and where it fails. Each verb reports through a {@link CommandReporter},
 * a failed restore as a warning: a reload that silently loses a project drops its person back to the picker.
 *
 * It registers with the tool's scope beside its `Session`, so leaving the tool cancels what it runs and forgets what it
 * holds, while the scope's release closes the openings on the backend.
 */
export class SessionRootResource<T> implements IScopedSession {
  /**
   * Whether the backend has been asked what the tool still holds, a restore cancelled or failed included, or a root
   * opened since, which a restore asked for meanwhile joins rather than runs.
   */
  @Observable()
  public isRestored: boolean = false;

  @Observable()
  private root: AsyncState<SessionSnapshot<T>> = AsyncState.idle<SessionSnapshot<T>>();

  private readonly session: Session;

  /**
   * @param log - The owning service's logger.
   * @param options - What is held and how it is let go and found again.
   * @param scopeService - The scope of the tool holding the root.
   * @param reporter - Reports each verb; built from the container's bus and application by default.
   */
  public constructor(
    log: Logger,
    private readonly options: ISessionRootOptions<T>,
    scopeService: SessionScopeService = inject(SessionScopeService),
    private readonly reporter: CommandReporter = new CommandReporter(log)
  ) {
    this.session = new Session(options.close, scopeService);
    scopeService.register(this);
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
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
   * Takes back what the tool still holds, unless a person's request has taken the lane.
   */
  @ExclusiveFlow("state")
  public *restore(): TFlow {
    const { restore, subject } = this.options;

    try {
      if (!restore) {
        return;
      }

      const completion: TCommandCompletion<SessionRestore<T>> = yield* this.reporter.run({
        name: `Restore ${subject}`,
        command: () => restore(this.session.tool),
        failure: `Could not restore the open ${subject}`,
        failureSeverity: ENotificationSeverity.WARNING,
      });

      if (completion.error) {
        this.root = this.root.asFailed(completion.error);
      } else {
        this.session.adopt(completion.result);
        this.root = this.root.asReady(completion.result);
      }
    } finally {
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
   */
  @LatestFlow("state")
  public *open<Args extends Array<unknown>>(
    label: string,
    command: (sessionId: SessionId, ...args: Args) => Promise<SessionSnapshot<T>>,
    ...args: Args
  ): TFlow {
    yield* this.replace(`Open ${this.options.subject}`, `Could not open the ${this.options.subject}`, label, () =>
      this.session.open(command, ...args)
    );
  }

  /**
   * Opens the held root again, joining an open or a refresh already under way; nothing while nothing is open.
   *
   * @param label - What is opened again, as the log lines and a failure name it.
   * @param command - The domain's open command, as {@link open} takes it.
   * @param args - The command's remaining arguments.
   */
  @ExclusiveFlow("state")
  public *refresh<Args extends Array<unknown>>(
    label: string,
    command: (sessionId: SessionId, ...args: Args) => Promise<SessionSnapshot<T>>,
    ...args: Args
  ): TFlow {
    if (this.root.value) {
      yield* this.replace(
        `Refresh ${this.options.subject}`,
        `Could not refresh the ${this.options.subject}`,
        label,
        () => this.session.open(command, ...args)
      );
    }
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

    const completion: TCommandCompletion<void> = yield* this.reporter.run({
      name: `Close ${this.options.subject}`,
      command: () => this.session.close(),
      failure: `Could not close the ${this.options.subject}`,
    });

    if (completion.error === null) {
      this.root = this.root.asIdle();
    } else {
      // A close that overtook an open keeps what that open kept on screen, which nothing is loading any more.
      this.root = held.isLoading ? held.asReady() : held;
    }

    return completion.error === null;
  }

  /** Cancels what runs and forgets what is held as the tool is left; the scope's release closes the openings. */
  @BoundAction()
  public retire(): void {
    cancelFlows(this);
    this.root = this.root.asIdle();
  }

  private *replace(name: string, failure: string, label: string, command: () => Promise<SessionSnapshot<T>>): TFlow {
    this.root = this.root.asLoading();

    try {
      const completion: TCommandCompletion<SessionSnapshot<T>> = yield* this.reporter.run({
        name,
        subject: label,
        command,
        failure,
      });

      this.root = completion.error ? this.root.asFailed(completion.error) : this.root.asReady(completion.result);
    } finally {
      this.isRestored = true;
    }
  }
}
