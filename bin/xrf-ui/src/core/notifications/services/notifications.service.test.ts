import { describe, expect, it } from "@jest/globals";
import { Container, EventBus } from "@wirestate/core";

import { emitNotification, ENotificationSeverity, INotification } from "@/core/notifications/lib";
import { NotificationsService } from "@/core/notifications/services/notifications.service";
import { EApplicationId } from "@/core/routing/application";
import { mockContainer, mockInjectedService } from "@/fixtures/utils/container";

describe("NotificationsService", () => {
  it("stamps a record and keeps the newest first", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.SUCCESS, source: EApplicationId.ARCHIVES_EXPLORER, title: "First" });
    service.push({ severity: ENotificationSeverity.ERROR, source: EApplicationId.ARCHIVES_EXPLORER, title: "Second" });

    expect(service.notifications.map((it: INotification) => it.title)).toEqual(["Second", "First"]);
    expect(service.notifications.every((it: INotification) => Boolean(it.id) && !it.isRead)).toBe(true);
    expect(new Set(service.notifications.map((it: INotification) => it.id)).size).toBe(2);
  });

  it("drops the oldest record rather than growing without bound", () => {
    const { service } = mockInjectedService(NotificationsService);

    for (let it = 0; it <= NotificationsService.LIMIT; it += 1) {
      service.push({
        severity: ENotificationSeverity.INFO,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: `Record ${it}`,
      });
    }

    expect(service.notifications).toHaveLength(NotificationsService.LIMIT);
    expect(service.notifications[0].title).toBe(`Record ${NotificationsService.LIMIT}`);
    expect(service.notifications.at(-1)?.title).toBe("Record 1");
  });

  it("badges the most urgent unread severity, not the newest one", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.ERROR, source: EApplicationId.ARCHIVES_EXPLORER, title: "Failed" });
    service.push({
      severity: ENotificationSeverity.SUCCESS,
      source: EApplicationId.ARCHIVES_EXPLORER,
      title: "Worked",
    });

    expect(service.unreadCount).toBe(2);
    expect(service.highestUnreadSeverity).toBe(ENotificationSeverity.ERROR);
  });

  it("has nothing to badge once everything is read", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.ERROR, source: EApplicationId.ARCHIVES_EXPLORER, title: "Failed" });
    service.markAllRead();

    expect(service.unreadCount).toBe(0);
    expect(service.highestUnreadSeverity).toBeNull();
    // Read, not removed - the panel is the record of what happened, and the badge is only the alert.
    expect(service.notifications).toHaveLength(1);
  });

  it("clears everything on request", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({
      severity: ENotificationSeverity.INFO,
      source: EApplicationId.ARCHIVES_EXPLORER,
      title: "Something",
    });
    service.clear();

    expect(service.notifications).toHaveLength(0);
  });

  it("records what the event bus delivers, which is how every editor reaches it", () => {
    const container: Container = mockContainer([NotificationsService]);

    container.provision();

    const service: NotificationsService = container.get(NotificationsService);

    emitNotification(container.get(EventBus), {
      details: "C:\\out",
      severity: ENotificationSeverity.SUCCESS,
      source: EApplicationId.ARCHIVES_EXPLORER,
      title: "Extracted textures",
    });

    expect(service.notifications).toHaveLength(1);
    expect(service.notifications[0].title).toBe("Extracted textures");
    expect(service.notifications[0].details).toBe("C:\\out");
  });
});
