import { describe, expect, it } from "@jest/globals";
import { LevelDecompileResult } from "@xrf/ipc/types/xrf-app-levels";
import { LevelDecompileReport, LevelDecompileRequest, LevelSceneRefusal } from "@xrf/ipc/types/xrf-rawdata";

import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

import { describeDecompileOutcome } from "./describe-decompile-outcome";

const REQUEST: LevelDecompileRequest = {
  source: "C:\\Games\\CoP\\gamedata\\levels\\mp_darkvalley",
  destination: "C:\\SDK\\rawdata\\levels\\mp_darkvalley.level",
  objects: null,
  textures: null,
  rootSpawn: null,
  surfaces: "infer",
  gameMaterials: null,
  compilerShaders: null,
  libraries: [],
  isForced: false,
};

function result(refused: Array<LevelSceneRefusal> = []): LevelDecompileResult {
  return { outcome: "completed", report: { refused } as LevelDecompileReport };
}

function outcome(patch: Partial<IJobOutcome<LevelDecompileResult>>): IJobOutcome<LevelDecompileResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeDecompileOutcome", () => {
  it("reports a clean decompile as a success naming the level and the scene", () => {
    const notice: IJobNotice = describeDecompileOutcome(REQUEST, outcome({ result: result() }));

    expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notice.details).toContain(REQUEST.source);
    expect(notice.details).toContain(REQUEST.destination);
  });

  it("warns where the scene left something out, and says how much", () => {
    const refusal: LevelSceneRefusal = { file: "level.spawn", object: "team_base", reason: "Unknown section" };
    const notice: IJobNotice = describeDecompileOutcome(REQUEST, outcome({ result: result([refusal, refusal]) }));

    expect(notice.severity).toBe(ENotificationSeverity.WARNING);
    expect(notice.details).toContain("2 pieces");
  });

  it("reports a cancelled decompile as having written nothing", () => {
    const notice: IJobNotice = describeDecompileOutcome(
      REQUEST,
      outcome({ result: { outcome: "cancelled", report: null } })
    );

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.details).toContain("nothing was written");
  });

  it("reports a failure with the reason", () => {
    const notice: IJobNotice = describeDecompileOutcome(
      REQUEST,
      outcome({ error: new Error("holds none of the level files") })
    );

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    expect(notice.details).toContain("holds none of the level files");
  });
});
