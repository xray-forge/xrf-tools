import { describe, expect, it } from "@jest/globals";
import { autorun, IReactionDisposer } from "@wirestate/mobx";

import { ENotificationSeverity, INotification } from "@/core/notifications/lib";
import { NotificationsService } from "@/core/notifications/services/notifications.service";
import { EApplicationId } from "@/core/routing/application";
import { mockInjectedService } from "@/fixtures/utils/container";

const SOURCE: EApplicationId = EApplicationId.SPRITE_EQUIPMENT_EDITOR;

describe("NotificationsService dev traces", () => {
  it("keeps traces out of the list real outcomes live in", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: "grid recomputed" });
    service.push({ severity: ENotificationSeverity.ERROR, source: SOURCE, title: "Pack failed" });

    expect(service.notifications.map((it: INotification) => it.title)).toEqual(["Pack failed"]);
    expect(service.devNotifications.map((it: INotification) => it.title)).toEqual(["grid recomputed"]);
  });

  it("records traces whatever the dev mode switch says, so it can be turned on afterwards", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: "grid recomputed" });

    // Nothing here consults a setting: the switch decides what is displayed, never what is kept.
    expect(service.devNotifications).toHaveLength(1);
  });

  it("gives traces their own budget, so a chatty one cannot evict a real outcome", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.ERROR, source: SOURCE, title: "Pack failed" });

    for (let it = 0; it <= NotificationsService.DEV_LIMIT; it += 1) {
      service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: `trace ${it}` });
    }

    expect(service.devNotifications).toHaveLength(NotificationsService.DEV_LIMIT);
    expect(service.notifications).toHaveLength(1);
  });

  it("leaves the badge alone, so traces cannot keep it permanently lit", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: "grid recomputed" });

    expect(service.unreadCount).toBe(0);
    expect(service.highestUnreadSeverity).toBeNull();
  });

  it("merges both lists into one chronology for the dev mode reading", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.ERROR, source: SOURCE, title: "first" });
    service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: "second" });
    service.push({ severity: ENotificationSeverity.SUCCESS, source: SOURCE, title: "third" });

    expect(service.allNotifications.map((it: INotification) => it.title)).toEqual(["third", "second", "first"]);
  });

  it("tracks both lists, so the panel re-renders whichever one changed", () => {
    const { service } = mockInjectedService(NotificationsService);

    const seen: Array<number> = [];

    const dispose: IReactionDisposer = autorun(() => seen.push(service.allNotifications.length));

    service.push({ severity: ENotificationSeverity.ERROR, source: SOURCE, title: "Pack failed" });
    service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: "trace" });

    dispose();

    expect(seen).toEqual([0, 1, 2]);
  });

  it("hands back one array until something changes, rather than re-sorting per read", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.ERROR, source: SOURCE, title: "Pack failed" });

    // Computed values only cache while something observes them, which in the application is the panel
    // rendering as an observer.
    const dispose: IReactionDisposer = autorun(() => service.allNotifications);

    const first: Array<INotification> = service.allNotifications;

    expect(service.allNotifications).toBe(first);

    service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: "trace" });

    expect(service.allNotifications).not.toBe(first);

    dispose();
  });

  it("clears both lists at once", () => {
    const { service } = mockInjectedService(NotificationsService);

    service.push({ severity: ENotificationSeverity.DEV, source: SOURCE, title: "trace" });
    service.push({ severity: ENotificationSeverity.ERROR, source: SOURCE, title: "Pack failed" });

    service.clear();

    expect(service.notifications).toHaveLength(0);
    expect(service.devNotifications).toHaveLength(0);
  });
});
