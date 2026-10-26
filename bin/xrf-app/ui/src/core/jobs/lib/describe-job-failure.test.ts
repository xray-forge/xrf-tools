import { describe, expect, it } from "@jest/globals";

import { ENotificationSeverity } from "@/core/notifications/lib";

import { describeJobFailure } from "./describe-job-failure";

describe("describeJobFailure", () => {
  it("names what the job was about, then why it failed, as an error", () => {
    expect(describeJobFailure("Could not verify configs", "C:\\configs", new Error("unreadable"))).toEqual({
      details: "C:\\configs\nunreadable",
      severity: ENotificationSeverity.ERROR,
      title: "Could not verify configs",
    });
  });
});
