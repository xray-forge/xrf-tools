import { describe, expect, it } from "@jest/globals";
import { SpawnBuildResult } from "@xrf/ipc/types/xrf-app-spawn";
import { Finding } from "@xrf/ipc/types/xrf-report";
import { SpawnRootRequest } from "@xrf/ipc/types/xrf-spawn-builder";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { mockSpawnBuildResult } from "@/tools/spawn-compiler/fixtures/spawn-build.mocks";

import { describeSpawnBuildOutcome } from "./describe-spawn-build-outcome";

const REQUEST: SpawnRootRequest = {
  gamedata: "D:\\qa\\gamedata",
  levelsRoot: "D:\\qa\\gamedata\\levels",
  levels: ["zaton", "labx8"],
  destination: "D:\\qa\\spawn",
  name: "qa_levels",
  options: { start: "labx8", isSeparatorStrict: false },
  isForced: false,
};

function outcome(patch: Partial<IJobOutcome<SpawnBuildResult>>): IJobOutcome<SpawnBuildResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeSpawnBuildOutcome", () => {
  it("reports a clean build as a success naming the spawn, its levels and what it counts", () => {
    const notice: IJobNotice = describeSpawnBuildOutcome(REQUEST, outcome({ result: mockSpawnBuildResult() }));

    expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notice.details).toContain("D:\\qa\\spawn\\qa_levels.spawn");
    expect(notice.details).toContain("2 levels: zaton, labx8.");
    expect(notice.details).toContain("1,234 objects, 56 artefact points, 7 patrols.");
  });

  it("warns where the build reported findings", () => {
    const finding: Finding = {
      ruleId: "spawn.connection.skipped",
      severity: "warning",
      subject: "zaton",
      message: "Graph point zat_to_jup connects to level jupiter, which is not built; the connection is left out",
    };
    const notice: IJobNotice = describeSpawnBuildOutcome(REQUEST, outcome({ result: mockSpawnBuildResult([finding]) }));

    expect(notice.severity).toBe(ENotificationSeverity.WARNING);
    expect(notice.details).toContain("1 findings reported.");
  });

  it("reports a build stopped before writing as having written nothing", () => {
    const notice: IJobNotice = describeSpawnBuildOutcome(
      REQUEST,
      outcome({ result: { outcome: "cancelled", spawn: null } })
    );

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.details).toContain("nothing was written");
  });

  it("reports a refused build with its reason", () => {
    const reason: string = "Level changer zat_to_jup on level zaton leads to level jupiter, which is not built";
    const notice: IJobNotice = describeSpawnBuildOutcome(REQUEST, outcome({ error: new Error(reason) }));

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    expect(notice.details).toContain(reason);
  });
});
