import { EventBus } from "@wirestate/core";

import { EMIT_NOTIFICATION_EVENT, INotificationPayload } from "@/core/notifications/lib/notifications-types";

/**
 * Record an outcome in the notification centre.
 *
 * @param bus - Event bus that delivers the notification.
 * @param payload - Notification details supplied by the emitter.
 */
export function emitNotification(bus: EventBus, payload: INotificationPayload): void {
  bus.emit<INotificationPayload>(EMIT_NOTIFICATION_EVENT, payload);
}
