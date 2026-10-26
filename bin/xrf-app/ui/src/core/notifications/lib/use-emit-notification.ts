import { EventBus } from "@wirestate/core";
import { useInjection } from "@wirestate/react";
import { useCallback } from "react";

import { emitNotification } from "@/core/notifications/lib/emit-notification";
import { INotificationPayload } from "@/core/notifications/lib/notifications-types";

/**
 * Function that emits a notification from a component.
 *
 * @param payload - Notification details supplied by the component.
 */
export type TEmitNotification = (payload: INotificationPayload) => void;

/**
 * Create a stable function that emits notifications from a component.
 *
 * @returns A function that emits the supplied notification payload.
 */
export function useEmitNotification(): TEmitNotification {
  const eventBus: EventBus = useInjection(EventBus);

  return useCallback((payload: INotificationPayload) => emitNotification(eventBus, payload), [eventBus]);
}
