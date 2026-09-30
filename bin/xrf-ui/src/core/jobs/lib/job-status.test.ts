import { describe, expect, it } from "@jest/globals";

import { JobDescription } from "@/core/ipc/types/xrf-app";

import { describeJobStatus } from "./job-status";

function job(patch: Partial<JobDescription>): JobDescription {
  return { conclusion: null, isCancelRequested: false, ...patch } as JobDescription;
}

describe("describeJobStatus", () => {
  it("says a live job is running, or stopping once asked to", () => {
    expect(describeJobStatus(job({}))).toEqual({ color: "text.secondary", label: "running" });
    expect(describeJobStatus(job({ isCancelRequested: true }))).toEqual({ color: "text.secondary", label: "stopping" });
  });

  it("paints how a job ended, a stopped one neutral", () => {
    expect(describeJobStatus(job({ conclusion: "completed" })).color).toBe("success.main");
    expect(describeJobStatus(job({ conclusion: "failed" })).color).toBe("error.main");
    expect(describeJobStatus(job({ conclusion: "cancelled" }))).toEqual({
      color: "text.secondary",
      label: "cancelled",
    });
  });
});
