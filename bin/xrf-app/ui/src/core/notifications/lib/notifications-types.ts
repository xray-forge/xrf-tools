import { EventType } from "@wirestate/core";

/**
 * The one event the notification centre listens for.
 */
export const EMIT_NOTIFICATION_EVENT: EventType = Symbol("@/notification/emit");

/** Notification urgency, ordered by `NOTIFICATION_SEVERITY_RANK` rather than declaration order. */
export enum ENotificationSeverity {
  /** Diagnostic trace for application developers. */
  DEV = "dev",
  /** Successful outcome. */
  SUCCESS = "success",
  /** Informational outcome. */
  INFO = "info",
  /** Outcome that may require attention. */
  WARNING = "warning",
  /** Failed outcome. */
  ERROR = "error",
}

/**
 * Rank used to colour the unread badge when records of several severities are waiting.
 *
 * The badge shows one colour for a mixed pile, so it has to be the most urgent one - a failure hidden
 * behind four successes is exactly the case the panel exists for.
 */
export const NOTIFICATION_SEVERITY_RANK: Record<ENotificationSeverity, number> = {
  [ENotificationSeverity.DEV]: -1,
  [ENotificationSeverity.SUCCESS]: 0,
  [ENotificationSeverity.INFO]: 1,
  [ENotificationSeverity.WARNING]: 2,
  [ENotificationSeverity.ERROR]: 3,
};

/**
 * Notification details supplied by an emitter.
 *
 * The service stamps every other record field.
 */
export interface INotificationPayload {
  /** Urgency of the recorded outcome. */
  severity: ENotificationSeverity;
  /** Tool that produced the outcome. */
  source: string;
  /** Short outcome summary shown in the collapsed row. */
  title: string;
  /** Longer context, such as a path, error message, or count, shown when the row is expanded. */
  details?: string;
}

/** Notification record stamped and stored by the notification service. */
export interface INotification extends INotificationPayload {
  /** Monotonically increasing record identifier. */
  id: string;
  /** Creation time as milliseconds since the Unix epoch. */
  createdAt: number;
  /** Whether the notification has been acknowledged in the panel. */
  isRead: boolean;
}
