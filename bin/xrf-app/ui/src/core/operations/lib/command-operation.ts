import { BoundAction, Computed, makeObservable, Observable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { TFlow } from "@xrf/flow";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { CommandReporter, ICommandReport, TCommandCompletion } from "@/core/operations/lib/command-reporter";

/**
 * One native command's state as a view shows it, for a command that is not a job: what it last answered, whether it
 * runs, and why it failed. The sibling of `JobOperation`.
 *
 * The owner runs it with `yield*` inside its own flow, which decides what supersedes what; the operation keeps the
 * previous answer while a run goes on and where one fails, and reports through a {@link CommandReporter}.
 */
export class CommandOperation<T> {
  @Observable()
  private answer: AsyncState<T> = AsyncState.idle<T>();

  /**
   * @param log - The owning service's logger.
   * @param reporter - Reports each run; built from the container's bus and application by default.
   */
  public constructor(
    log: Logger,
    private readonly reporter: CommandReporter = new CommandReporter(log)
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /** The command's lifecycle and last answer. */
  @Computed()
  public get state(): AsyncState<T> {
    return this.answer;
  }

  @Computed()
  public get value(): Nullable<T> {
    return this.answer.value;
  }

  @Computed()
  public get isLoading(): boolean {
    return this.answer.isLoading;
  }

  @Computed()
  public get error(): Nullable<Error> {
    return this.answer.error;
  }

  /** Forgets the last outcome, leaving a run in progress alone. */
  @BoundAction()
  public reset(): void {
    if (!this.answer.isLoading) {
      this.answer = this.answer.asIdle();
    }
  }

  /**
   * Runs the command inside the owner's flow, so superseding or deactivating the owner cannot publish a late answer.
   *
   * @param report - The command and how it is reported.
   * @returns How the command ended.
   */
  public *run(report: ICommandReport<T>): TFlow<TCommandCompletion<T>> {
    this.answer = this.answer.asLoading();

    try {
      const completion: TCommandCompletion<T> = yield* this.reporter.run(report);

      this.answer =
        completion.error === null ? this.answer.asReady(completion.result) : this.answer.asFailed(completion.error);

      return completion;
    } finally {
      // A cancelled run answered nothing, so what it kept is shown as it was before.
      if (this.answer.isLoading) {
        this.answer = this.answer.asIdle(this.answer.value);
      }
    }
  }
}
