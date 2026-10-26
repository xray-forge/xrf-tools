import { EventBus, inject } from "@wirestate/core";
import { transformError } from "@xrf/errors";
import { call, TFlow } from "@xrf/flow";
import { formatDuration } from "@xrf/format";
import { SESSION_TOOL } from "@xrf/ipc/session";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";

/** A finished command: its answer, or why it failed. */
export type TCommandCompletion<T> = { result: T; error: null } | { result: null; error: Error };

/** What a finished command tells the notification centre; the reporter adds its severity and source. */
export interface ICommandNotice {
  title: string;
  details?: string;
}

/** One native command as a person sees it run: what it is called, what it acts on, and what it says when done. */
export interface ICommandReport<T> {
  /** What the command does, as its log lines name it: `Export externs manifest`. */
  name: string;
  /** What it acts on, logged and leading a failure's details: a path, a root; null where nothing names it. */
  subject?: Nullable<string>;
  /** Starts the native work. */
  command: () => Promise<T>;
  /** The failure notice's title. */
  failure: string;
  /** How urgent a failure is: an error, unless the person did not ask for the command. */
  failureSeverity?: ENotificationSeverity;
  /** The notice a success raises, or none. */
  success?: (result: T) => ICommandNotice;
}

/**
 * Runs native commands inside a service's flow and reports each the same way: a start line, a done or failed line
 * with the time taken, the failure as an `Error`, and a notice from the application the container is scoped to.
 *
 * Reporting never throws: the completion says how the command ended. A flow cancelled while the command runs reports
 * nothing, as a superseded run publishes nothing.
 */
export class CommandReporter {
  /**
   * @param log - The owning service's logger, so the lines name the service.
   * @param eventBus - The bus the notices are told on.
   * @param source - The application the notices name: the container's tool, which is the application's id.
   */
  public constructor(
    private readonly log: Logger,
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly source: string = inject(SESSION_TOOL)
  ) {}

  /**
   * Runs one command inside the caller's flow.
   *
   * @param report - The command and how it is reported.
   * @returns How the command ended.
   */
  public *run<T>(report: ICommandReport<T>): TFlow<TCommandCompletion<T>> {
    const { name, subject = null, command, failure, failureSeverity = ENotificationSeverity.ERROR, success } = report;
    const subjects: Array<string> = subject === null ? [] : [subject];
    const timer: Timer = new Timer();

    this.log.info(`${name}:`, ...subjects);

    try {
      const result: T = yield* call(command());

      this.log.info(`${name} done:`, ...subjects, "in", formatDuration(timer.elapsed()));

      if (success) {
        this.notify(ENotificationSeverity.SUCCESS, success(result));
      }

      return { result, error: null };
    } catch (caught: unknown) {
      const error: Error = transformError(caught);

      this.log.error(`${name} failed:`, ...subjects, "after", formatDuration(timer.elapsed()), error);
      this.notify(failureSeverity, {
        details: subject === null ? error.message : `${subject}\n${error.message}`,
        title: failure,
      });

      return { result: null, error };
    }
  }

  private notify(severity: ENotificationSeverity, notice: ICommandNotice): void {
    emitNotification(this.eventBus, { ...notice, severity, source: this.source });
  }
}
