import { describe, expect, it } from "@jest/globals";

import { JobConclusion, JobDescription } from "@/core/bindings/types/xrf-app";
import { listHeldLeases, summarizeJobKinds } from "@/core/jobs/lib/job-listing";

function mockJob(overrides: Partial<JobDescription> = {}): JobDescription {
  return {
    id: "job",
    kind: "archives.pack",
    leaseKeys: [],
    request: null,
    isCancelRequested: false,
    progress: null,
    conclusion: null,
    error: null,
    result: null,
    duration: 0,
    startedAt: 0,
    ...overrides,
  } as JobDescription;
}

describe("job history", () => {
  it("counts a run of each ending under its own name", () => {
    const endings: Array<JobConclusion> = ["completed", "cancelled", "failed"];
    const summaries = summarizeJobKinds(
      endings.map((conclusion: JobConclusion, index: number) =>
        mockJob({ id: String(index), conclusion, duration: (index + 1) * 1000 })
      )
    );

    expect(summaries).toHaveLength(1);
    expect(summaries[0]).toMatchObject({
      kind: "archives.pack",
      runs: 3,
      completed: 1,
      cancelled: 1,
      failed: 1,
      duration: 6000,
      slowest: 3000,
    });
  });

  it("counts a running job without letting it into the durations", () => {
    // A run still going has no total, and adding its elapsed time would make an average that moves while nobody
    // does anything.
    const summaries = summarizeJobKinds([
      mockJob({ id: "a", conclusion: "completed", duration: 1000 }),
      mockJob({ id: "b", duration: 9999 }),
    ]);

    expect(summaries[0]).toMatchObject({ runs: 2, completed: 1, duration: 1000, slowest: 1000 });
  });

  it("lists what the running jobs hold and ignores what the finished ones held", () => {
    const leases = listHeldLeases([
      mockJob({ id: "a", leaseKeys: ["archives:out.db"] }),
      mockJob({ id: "b", kind: "configs.format", leaseKeys: ["configs:gamedata"], conclusion: "completed" }),
      mockJob({ id: "c", kind: "spawn.pack", leaseKeys: ["spawn:all.spawn", "spawn:level"] }),
    ]);

    expect(leases).toEqual([
      { key: "archives:out.db", id: "a", kind: "archives.pack" },
      { key: "spawn:all.spawn", id: "c", kind: "spawn.pack" },
      { key: "spawn:level", id: "c", kind: "spawn.pack" },
    ]);
  });
});
