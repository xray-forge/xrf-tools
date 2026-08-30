import { describe, expect, it } from "@jest/globals";

import { describePackOutcome } from "@/applications/archives-packer/lib/describe-pack-outcome";
import { ArchivePackConfig, ArchivePackResult } from "@/core/bindings/types/xrf-pack";
import { IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";

const CONFIG: ArchivePackConfig = {
  source: "C:\\work\\gamedata",
  destination: "C:\\work\\db",
  name: "gamedata",
} as ArchivePackConfig;

function result(patch: Partial<ArchivePackResult> = {}): ArchivePackResult {
  return { outcome: "completed", volumes: [], volumesOpened: [], ...patch } as ArchivePackResult;
}

function outcome(patch: Partial<IJobOutcome<ArchivePackResult>>): IJobOutcome<ArchivePackResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describePackOutcome", () => {
  it("reports a finished pack as a success naming both ends", () => {
    const notification: INotificationPayload = describePackOutcome(CONFIG, outcome({ result: result() }));

    expect(notification.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notification.details).toContain("C:\\work\\gamedata");
    expect(notification.details).toContain("C:\\work\\db");
  });

  it("reports a failure with the reason rather than the destination", () => {
    const notification: INotificationPayload = describePackOutcome(
      CONFIG,
      outcome({ error: new Error("volume cap refuses particles.xr") })
    );

    expect(notification.severity).toBe(ENotificationSeverity.ERROR);
    expect(notification.details).toContain("volume cap refuses particles.xr");
  });

  it("warns about a cancelled pack and names every volume it left behind", () => {
    // The contract this exists for. Packing opens each volume by truncating whatever stood there, so a stopped run
    // leaves an unusable set and removes nothing - these paths are the only way the user learns what to clean up.
    const notification: INotificationPayload = describePackOutcome(
      CONFIG,
      outcome({
        isCancelRequested: true,
        result: result({
          outcome: "cancelled",
          volumes: ["C:\\work\\db\\gamedata.db0"],
          volumesOpened: ["C:\\work\\db\\gamedata.db0", "C:\\work\\db\\gamedata.db1"],
        }),
      })
    );

    expect(notification.severity).toBe(ENotificationSeverity.WARNING);
    expect(notification.details).toContain("C:\\work\\db\\gamedata.db0");
    expect(notification.details).toContain("C:\\work\\db\\gamedata.db1");
  });

  it("reads the outcome off the result rather than off the cancel request", () => {
    // Cancelling is a request the run may finish ahead of. A pack that completed after the control was pressed still
    // produced a whole set, and calling that "stopped" would send the user hunting for wreckage that is not there.
    const notification: INotificationPayload = describePackOutcome(
      CONFIG,
      outcome({ isCancelRequested: true, result: result() })
    );

    expect(notification.severity).toBe(ENotificationSeverity.SUCCESS);
  });
});
