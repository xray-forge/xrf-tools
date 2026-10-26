import { describe, expect, it } from "@jest/globals";

import { ENotificationSeverity, INotification } from "@/core/notifications/lib";
import { INotificationEntry, toNotificationEntries } from "@/core/shell/panel/notifications/notification-list";

/**
 * A stored record, stamped as the service would stamp it.
 *
 * @param id - Record identifier.
 * @param createdAt - Creation time as milliseconds since the Unix epoch.
 * @param overrides - Fields that matter to the case under test.
 * @returns The record.
 */
function record(id: string, createdAt: number, overrides: Partial<INotification> = {}): INotification {
  return {
    createdAt,
    id,
    isRead: false,
    severity: ENotificationSeverity.INFO,
    source: "archives-explorer",
    title: "Rebuilt index",
    ...overrides,
  };
}

describe("toNotificationEntries", () => {
  it("collapses a run of the same event into one counted row", () => {
    const now: number = Date.now();
    const entries: Array<INotificationEntry> = toNotificationEntries([
      record("3", now),
      record("2", now - 1000),
      record("1", now - 2000),
    ]);

    expect(entries).toHaveLength(1);
    expect(entries[0].repeatCount).toBe(3);
    // The newest of the run stands for it, so the row reads as the most recent time it happened.
    expect(entries[0].notification.id).toBe("3");
  });

  it("keeps events apart when anything a row shows about them differs", () => {
    const now: number = Date.now();
    const entries: Array<INotificationEntry> = toNotificationEntries([
      record("3", now, { details: "4128 entries" }),
      record("2", now - 1000),
      record("1", now - 2000, { severity: ENotificationSeverity.WARNING }),
    ]);

    expect(entries).toHaveLength(3);
  });
});
