import { describe, expect, it } from "@jest/globals";

import { EXrayEngine, EXrayEngineEvidence } from "@/core/ipc/types/xrf-engine-target";

import { describeEngineResolution, toAutoEngineLabel } from "./engine-target-label";

describe("engine target labels", () => {
  it("says what Auto found and why", () => {
    expect(
      toAutoEngineLabel(
        { engine: EXrayEngine.EXTENDED, evidence: EXrayEngineEvidence.ANOMALY_EXECUTABLES, subject: "bin" },
        false
      )
    ).toBe("Auto: Extended (Anomaly executables found)");
    expect(
      toAutoEngineLabel({ engine: EXrayEngine.VANILLA, evidence: EXrayEngineEvidence.NO_SIGNS, subject: null }, false)
    ).toBe("Auto: Vanilla (no Anomaly or Atmosfear signs)");
  });

  it("says Auto alone while detection runs or where it could not answer", () => {
    expect(toAutoEngineLabel(null, true)).toBe("Auto: detecting");
    expect(toAutoEngineLabel(null, false)).toBe("Auto");
  });

  it("describes every evidence the backend can give", () => {
    for (const evidence of Object.values(EXrayEngineEvidence)) {
      expect(describeEngineResolution({ engine: EXrayEngine.EXTENDED, evidence, subject: null })).toMatch(
        /^Extended \(.+\)$/
      );
    }
  });
});
