import { EventBus, inject, Injectable, OnDeactivation, OnProvision } from "@wirestate/core";
import { BoundAction, makeObservable } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { APPLICATION_SOURCE } from "@/core/routing/application";
import { Logger } from "@/lib/logging";

/**
 * Records the failures nothing else reports.
 */
@Injectable()
export class ErrorCaptureService {
  /** Logger scoped to the error capture service. */
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Set while a capture is being recorded, so a failure in that path cannot re-enter and loop. */
  private isRecording: boolean = false;

  /**
   * Create an error capture service.
   *
   * @param eventBus - Event bus that delivers captured failures to the notification service.
   */
  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {
    makeObservable(this);
  }

  /**
   * Register global error and rejection listeners when the service is provisioned.
   */
  @OnProvision()
  public onProvision(): void {
    window.addEventListener("error", this.onWindowError);
    window.addEventListener("unhandledrejection", this.onUnhandledRejection);
  }

  /**
   * Remove global error and rejection listeners when the service is deactivated.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    window.removeEventListener("error", this.onWindowError);
    window.removeEventListener("unhandledrejection", this.onUnhandledRejection);
  }

  /**
   * Record an uncaught window error when it contains useful failure details.
   *
   * @param event - Uncaught window error event.
   */
  @BoundAction()
  public onWindowError(event: ErrorEvent): void {
    // Resource load failures reach the same event without an `error`, and say nothing worth recording.
    if (!event.error && !event.message) {
      return;
    }

    const where: string = event.filename ? `${event.filename}:${event.lineno}:${event.colno}` : "unknown location";

    this.record(event.error ? transformError(event.error).message : event.message, where);
  }

  /**
   * Record an unhandled promise rejection.
   *
   * @param event - Unhandled promise rejection event.
   */
  @BoundAction()
  public onUnhandledRejection(event: PromiseRejectionEvent): void {
    this.record(transformError(event.reason).message, "unhandled rejection");
  }

  /**
   * Record one captured failure, guarding against the loop where recording it fails again.
   *
   * @param message - Failure message to record.
   * @param where - Location or failure context shown in the notification details.
   */
  private record(message: string, where: string): void {
    if (this.isRecording) {
      return;
    }

    this.isRecording = true;

    try {
      emitNotification(this.eventBus, {
        details: where,
        severity: ENotificationSeverity.DEV,
        source: APPLICATION_SOURCE,
        title: message,
      });
    } finally {
      this.isRecording = false;
    }
  }
}
