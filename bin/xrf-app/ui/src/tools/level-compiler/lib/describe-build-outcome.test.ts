import { describe, expect, it } from "@jest/globals";
import { LevelBuildResult } from "@xrf/ipc/types/xrf-app-levels";
import { LevelBuildReport, LevelBuildRequest, LevelSceneRefusal } from "@xrf/ipc/types/xrf-rawdata";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

import { describeBuildOutcome } from "./describe-build-outcome";

const REQUEST: LevelBuildRequest = {
  source: "C:\\SDK\\rawdata\\levels\\mp_darkvalley.level",
  destination: "C:\\work\\levels\\mp_darkvalley",
  objects: ["C:\\SDK\\rawdata\\objects"],
  gameMaterials: "C:\\Games\\CoP\\gamedata\\gamemtl.xr",
  textures: null,
  isProofLighting: true,
  impostorLimit: "extended",
  isForced: false,
};

function result(refused: Array<LevelSceneRefusal> = []): LevelBuildResult {
  return { outcome: "completed", report: { files: [], refused } as unknown as LevelBuildReport, compile: null };
}

function outcome(patch: Partial<IJobOutcome<LevelBuildResult>>): IJobOutcome<LevelBuildResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeBuildOutcome", () => {
  it("reports a clean build as a success naming the scene and the level", () => {
    const notice: IJobNotice = describeBuildOutcome(REQUEST, outcome({ result: result() }));

    expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notice.details).toContain(REQUEST.source);
    expect(notice.details).toContain(REQUEST.destination);
  });

  it("warns where the build refused a part", () => {
    const refusal: LevelSceneRefusal = { file: "level.spawn", object: "meshes\\brkbl#0", reason: "Error face found" };
    const notice: IJobNotice = describeBuildOutcome(REQUEST, outcome({ result: result([refusal]) }));

    expect(notice.severity).toBe(ENotificationSeverity.WARNING);
    expect(notice.details).toContain("1 parts were refused");
  });

  it("reports a cancelled build as having written nothing", () => {
    const notice: IJobNotice = describeBuildOutcome(
      REQUEST,
      outcome({ result: { outcome: "cancelled", report: null, compile: null } })
    );

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.details).toContain("nothing was written");
  });

  it("reports a compile stopped after the build as keeping what was written", () => {
    const notice: IJobNotice = describeBuildOutcome(
      REQUEST,
      outcome({ result: { ...result(), outcome: "cancelled" } })
    );

    expect(notice.title).toBe("Stopped compiling the level");
    expect(notice.details).toContain("what it wrote so far stays");
  });

  it("reports a failure with the reason", () => {
    const notice: IJobNotice = describeBuildOutcome(REQUEST, outcome({ error: new Error("undefined game material") }));

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    expect(notice.details).toContain("undefined game material");
  });
});
