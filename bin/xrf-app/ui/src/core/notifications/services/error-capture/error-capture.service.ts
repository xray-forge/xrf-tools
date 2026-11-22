import { EventBus, inject, Injectable, OnDeactivation, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction } from "@wirestate/mobx";
import { transformError } from "@xrf/errors";
import { ELogLevel, Logger } from "@xrf/logging";

import { APPLICATION_SOURCE, emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { ICapturedFailure } from "@/core/notifications/services/error-capture/error-capture.service.failure";

/**
 * Records the failures nothing else reports, as a notification and as an error record in the UI log.
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
  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  /**
   * Register global error and rejection listeners when the service is provisioned.
   */
  @OnProvision()
  public onProvision(provisionId: ProvisionId): void {
    this.log.info("Provisioning:", provisionId);

    window.addEventListener("error", this.onWindowError);
    window.addEventListener("unhandledrejection", this.onUnhandledRejection);
  }

  /**
   * Remove global error and rejection listeners when the service is deactivated.
   */
  @OnDeactivation()
  public onDeactivation(): void {
    this.log.info("Deactivating");

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

    this.record({
      what: `Uncaught error at ${where}`,
      details: where,
      message: event.error ? transformError(event.error).message : event.message,
      cause: event.error instanceof Error ? event.error : null,
    });
  }

  /**
   * Record an unhandled promise rejection.
   *
   * @param event - Unhandled promise rejection event.
   */
  @BoundAction()
  public onUnhandledRejection(event: PromiseRejectionEvent): void {
    this.record({
      what: "Unhandled rejection",
      details: "unhandled rejection",
      message: transformError(event.reason).message,
      cause: event.reason instanceof Error ? event.reason : null,
    });
  }

  /**
   * Record one captured failure as an error record in the UI log and a notification, guarding against the loop where
   * recording it fails again.
   *
   * @param failure - The failure a listener caught.
   */
  private record(failure: ICapturedFailure): void {
    if (this.isRecording) {
      return;
    }

    this.isRecording = true;

    try {
      // The browser has printed the failure to the console already, so it goes to the UI log alone.
      this.log.record(ELogLevel.ERROR, `${failure.what}:`, failure.cause ?? failure.message);

      emitNotification(this.eventBus, {
        details: failure.details,
        severity: ENotificationSeverity.DEV,
        source: APPLICATION_SOURCE,
        title: failure.message,
      });
    } finally {
      this.isRecording = false;
    }
  }
}
