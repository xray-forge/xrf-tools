import { beforeEach, describe, expect, it } from "@jest/globals";

import { JobProgress, ProgressLevel } from "@/core/bindings/types/xrf-job";
import { IJobProfile, JOB_PROFILES } from "@/core/jobs/metrics";
import { Nullable } from "@/lib/types/general";

function mockLevel(id: string, completed: number): ProgressLevel {
  return { id, label: null, completed, total: null, unit: "items" };
}

function mockReport(duration: number, levels: Array<ProgressLevel>): JobProgress {
  return { duration, levels, detail: null };
}

function mockProfileOf(id: string): IJobProfile {
  const found: Nullable<IJobProfile> = JOB_PROFILES.read(id);

  if (!found) {
    throw new Error(`No profile for '${id}'.`);
  }

  return found;
}

describe("JobProfileRecorder", () => {
  beforeEach(() => {
    JOB_PROFILES.reset();
  });

  it("attributes each slice of time to the phase that was running through it", () => {
    JOB_PROFILES.begin("a", "archives.pack");

    // The first report closes nothing: there is no earlier phase for its elapsed time to belong to.
    JOB_PROFILES.sample("a", mockReport(0, [mockLevel("read", 0)]));
    JOB_PROFILES.sample("a", mockReport(1000, [mockLevel("read", 100)]));
    JOB_PROFILES.sample("a", mockReport(1500, [mockLevel("write", 0)]));
    JOB_PROFILES.sample("a", mockReport(4500, [mockLevel("write", 300)]));

    const profile: IJobProfile = mockProfileOf("a");

    expect(profile.phases.map((it) => [it.id, it.duration, it.completed])).toEqual([
      ["read", 1500, 100],
      ["write", 3000, 300],
    ]);
  });

  it("uses the registry's clock rather than this window's, so a slow render cannot stretch a phase", () => {
    JOB_PROFILES.begin("a", "archives.pack");

    JOB_PROFILES.sample("a", mockReport(0, [mockLevel("read", 0)]));
    JOB_PROFILES.sample("a", mockReport(250, [mockLevel("read", 10)]));

    expect(mockProfileOf("a").duration).toBe(250);
    expect(mockProfileOf("a").phases[0].duration).toBe(250);
  });

  it("reads the innermost level, which is the one counting work rather than the phases around it", () => {
    JOB_PROFILES.begin("a", "archives.pack");

    JOB_PROFILES.sample("a", mockReport(0, [mockLevel("pack", 0), mockLevel("write", 0)]));
    JOB_PROFILES.sample("a", mockReport(1000, [mockLevel("pack", 0), mockLevel("write", 50)]));

    expect(mockProfileOf("a").phases.map((it) => it.id)).toEqual(["write"]);
  });

  it("reports the best rate seen between two reports, not the average over the run", () => {
    JOB_PROFILES.begin("a", "archives.pack");

    JOB_PROFILES.sample("a", mockReport(0, [mockLevel("write", 0)]));
    JOB_PROFILES.sample("a", mockReport(1000, [mockLevel("write", 100)]));
    JOB_PROFILES.sample("a", mockReport(2000, [mockLevel("write", 900)]));

    // 800 units in the second, against 100 in the first.
    expect(mockProfileOf("a").peakRate).toBe(800);
    // The unit comes from the phase that set the peak, because phases do not agree on one.
    expect(mockProfileOf("a").peakUnit).toBe("items");
  });

  it("measures the longest run of reports that moved nothing, which is what a stall looks like", () => {
    JOB_PROFILES.begin("a", "configs.verify");

    JOB_PROFILES.sample("a", mockReport(0, [mockLevel("verify", 0)]));
    JOB_PROFILES.sample("a", mockReport(500, [mockLevel("verify", 10)]));
    JOB_PROFILES.sample("a", mockReport(1500, [mockLevel("verify", 10)]));
    JOB_PROFILES.sample("a", mockReport(3500, [mockLevel("verify", 10)]));
    JOB_PROFILES.sample("a", mockReport(4000, [mockLevel("verify", 20)]));

    expect(mockProfileOf("a").longestStall).toBe(3000);
  });

  it("counts the reports it received, which is this window's share of the channel traffic", () => {
    JOB_PROFILES.begin("a", "configs.verify");

    JOB_PROFILES.sample("a", mockReport(0, [mockLevel("verify", 0)]));
    JOB_PROFILES.sample("a", mockReport(100, [mockLevel("verify", 1)]));

    expect(mockProfileOf("a").samples).toBe(2);
  });

  it("says when it joined a run already going, because the phases before that are missing", () => {
    JOB_PROFILES.begin("adopted", "archives.unpack", true);

    expect(mockProfileOf("adopted").isPartial).toBe(true);
    expect(JOB_PROFILES.read("adopted")?.isFinished).toBe(false);

    JOB_PROFILES.finish("adopted");

    expect(mockProfileOf("adopted").isFinished).toBe(true);
  });

  it("ignores a report for a run it was never told about, rather than inventing one", () => {
    JOB_PROFILES.sample("unknown", mockReport(100, [mockLevel("write", 10)]));

    expect(JOB_PROFILES.read("unknown")).toBeNull();
    expect(JOB_PROFILES.list()).toEqual([]);
  });

  it("keeps no more profiles than the backend keeps listings, so neither runs out before the other", () => {
    for (let index: number = 0; index < 25; index += 1) {
      JOB_PROFILES.begin(`job-${index}`, "archives.pack");
    }

    expect(JOB_PROFILES.list()).toHaveLength(20);
    expect(JOB_PROFILES.read("job-0")).toBeNull();
    expect(JOB_PROFILES.read("job-24")).not.toBeNull();
  });

  it("hands out copies, so a render cannot see a phase grow underneath it", () => {
    JOB_PROFILES.begin("a", "archives.pack");

    JOB_PROFILES.sample("a", mockReport(0, [mockLevel("write", 0)]));
    JOB_PROFILES.sample("a", mockReport(1000, [mockLevel("write", 10)]));

    const profile: IJobProfile = mockProfileOf("a");

    JOB_PROFILES.sample("a", mockReport(2000, [mockLevel("write", 20)]));

    expect(profile.phases[0].duration).toBe(1000);
    expect(mockProfileOf("a").phases[0].duration).toBe(2000);
  });
});
