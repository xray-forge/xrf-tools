import { describe, expect, it } from "@jest/globals";
import { CaptureOutcome } from "@xrf/ipc/types/xrf-app-render";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

import { describeLevelCaptureOutcome } from "./describe-level-capture-outcome";
import { ELevelCaptureMode } from "./level-capture";

const PATH: string = "appdata/level-viewer/screenshots/map_zaton-20261021-101500.png";

function outcome(patch: Partial<IJobOutcome<CaptureOutcome>>): IJobOutcome<CaptureOutcome> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeLevelCaptureOutcome", () => {
  it("reports a map by its size and a cubemap by its faces, both as a success naming the file", () => {
    const map: IJobNotice = describeLevelCaptureOutcome(
      ELevelCaptureMode.MAP,
      PATH,
      outcome({ result: { outcome: "completed", width: 2048, height: 1024 } })
    );
    const cube: IJobNotice = describeLevelCaptureOutcome(
      ELevelCaptureMode.CUBEMAP,
      PATH,
      outcome({ result: { outcome: "completed", width: 2048, height: 1536 } })
    );

    expect(map).toMatchObject({ details: PATH, severity: ENotificationSeverity.SUCCESS });
    expect(map.title).toBe("Captured a 2048 × 1024 level map");
    expect(cube.title).toBe("Captured a cubemap of 512 pixel faces");
  });

  it("reports a stopped capture as information, saying nothing was written", () => {
    const notice: IJobNotice = describeLevelCaptureOutcome(
      ELevelCaptureMode.CUBEMAP,
      PATH,
      outcome({ isCancelRequested: true, result: { outcome: "cancelled", width: 0, height: 0 } })
    );

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.details).toContain("every face is back");
  });

  it("reports a failure with the reason", () => {
    const notice: IJobNotice = describeLevelCaptureOutcome(
      ELevelCaptureMode.MAP,
      PATH,
      outcome({ error: new Error("the device was lost") })
    );

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    expect(notice.title).toBe("Could not capture the level map");
    expect(notice.details).toContain("the device was lost");
  });
});
