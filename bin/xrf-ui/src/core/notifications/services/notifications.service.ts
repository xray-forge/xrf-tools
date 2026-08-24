import { Injectable, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import {
  EMIT_NOTIFICATION_EVENT,
  ENotificationSeverity,
  INotification,
  INotificationPayload,
  NOTIFICATION_SEVERITY_RANK,
} from "@/core/notifications/lib";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * The application wide record of what commands did.
 */
@Injectable()
export class NotificationsService {
  /** Maximum number of user-facing notifications retained by the service. */
  public static readonly LIMIT: number = 200;

  /** Maximum number of developer traces retained separately by the service. */
  public static readonly DEV_LIMIT: number = 100;

  /** Logger scoped to the notification service. */
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Counter used to stamp unique, chronologically ordered record identifiers. */
  private nextId: number = 0;

  /** User-facing notifications in newest-first order. */
  @Observable()
  public notifications: Array<INotification> = [];

  /** Developer traces in newest-first order, recorded regardless of the dev mode setting. */
  @Observable()
  public devNotifications: Array<INotification> = [];

  /**
   * Both lists in one chronology, which is the reading dev mode is turned on for.
   *
   * Computed rather than a plain getter: it allocates and sorts, and a fresh array on every read is
   * one nobody can compare by reference.
   *
   * @returns User-facing notifications and developer traces in newest-first order.
   */
  @Computed()
  public get allNotifications(): Array<INotification> {
    // Ids are a monotonic counter, so they order records that share a millisecond.
    return [...this.notifications, ...this.devNotifications].sort(
      (first: INotification, second: INotification) => Number(second.id) - Number(first.id)
    );
  }

  /**
   * Count unread user-facing notifications for the panel badge.
   *
   * @returns Number of unread user-facing notifications.
   */
  @Computed()
  public get unreadCount(): number {
    return this.notifications.reduce((count: number, it: INotification) => (it.isRead ? count : count + 1), 0);
  }

  /**
   * Find the highest severity among unread user-facing notifications.
   *
   * @returns Highest unread severity, or `null` when every notification is read.
   */
  @Computed()
  public get highestUnreadSeverity(): Nullable<ENotificationSeverity> {
    let highest: Nullable<ENotificationSeverity> = null;

    for (const notification of this.notifications) {
      if (
        !notification.isRead &&
        (highest === null || NOTIFICATION_SEVERITY_RANK[notification.severity] > NOTIFICATION_SEVERITY_RANK[highest])
      ) {
        highest = notification.severity;
      }
    }

    return highest;
  }

  /**
   * Record an outcome raised anywhere in the application.
   *
   * Public as well as bus-driven so a test does not need a provisioned container to describe what the
   * store should do with a record.
   *
   * @param payload - Notification details to stamp and store.
   */
  @BoundAction()
  public push(payload: INotificationPayload): void {
    const notification: INotification = {
      ...payload,
      id: String((this.nextId += 1)),
      createdAt: Date.now(),
      isRead: false,
    };

    if (payload.severity === ENotificationSeverity.DEV) {
      this.devNotifications = [notification, ...this.devNotifications].slice(0, NotificationsService.DEV_LIMIT);

      return;
    }

    this.log.info("Notification pushed:", payload.title, notification);

    this.notifications = [notification, ...this.notifications].slice(0, NotificationsService.LIMIT);
  }

  /**
   * Clear the unread badge.
   *
   * Called by the panel while it is open, including for records that arrive while it is open - they
   * were on screen as they landed, so calling them unread would leave a badge nothing can dismiss.
   */
  @BoundAction()
  public markAllRead(): void {
    if (this.unreadCount) {
      this.log.info("Marking all notifications as read");
      this.notifications = this.notifications.map((it: INotification) => (it.isRead ? it : { ...it, isRead: true }));
    }
  }

  /**
   * Remove all user-facing notifications and developer traces.
   */
  @BoundAction()
  public clear(): void {
    this.log.info("Clear all notifications");
    this.notifications = [];
    this.devNotifications = [];
  }

  /**
   * Store a notification delivered after the root container is provisioned.
   *
   * @param event - Notification event delivered by the application event bus.
   */
  @OnEvent(EMIT_NOTIFICATION_EVENT)
  public onNotificationPush(event: WireEvent<INotificationPayload>): void {
    if (!event.payload) {
      return this.log.warn("Ignoring notification event with no payload");
    }

    this.push(event.payload);
  }
}
