import { describe, expect, it } from "@jest/globals";

import { describePatchOutcome } from "@/applications/archives-patcher/lib/describe-patch-outcome";
import { ArchivePatchChange, ArchivePatchConfig, ArchivePatchResult } from "@/core/bindings/types/xrf-pack";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

const CONFIG: ArchivePatchConfig = {
  input: "C:\\Games\\Anomaly",
  target: null,
  destination: "C:\\work\\patches",
  name: "patch_02",
} as ArchivePatchConfig;

function change(name: string): ArchivePatchChange {
  return { name, class: "modified", base: null, target: null };
}

function result(patch: Partial<ArchivePatchResult> = {}): ArchivePatchResult {
  return {
    outcome: "completed",
    added: [],
    modified: [],
    unchanged: 0,
    payloadsRead: 0,
    publication: { kind: "compared" },
    ...patch,
  } as ArchivePatchResult;
}

function outcome(patch: Partial<IJobOutcome<ArchivePatchResult>>): IJobOutcome<ArchivePatchResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describePatchOutcome", () => {
  it("reports a published patch as a success naming its volumes", () => {
    const notification: IJobNotice = describePatchOutcome(
      CONFIG,
      outcome({
        result: result({
          modified: [change("configs\\system.ltx")],
          publication: { kind: "published", volumes: ["C:\\work\\patches\\patch_02.db"] } as never,
        }),
      })
    );

    expect(notification.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notification.title).toBe("Published patch");
    expect(notification.details).toContain("C:\\work\\patches\\patch_02.db");
  });

  it("distinguishes two worlds that agree from a preview that wrote nothing", () => {
    // Both leave the destination empty, so the outcome has to come off the publication rather than off a volume list.
    const agreed: IJobNotice = describePatchOutcome(
      CONFIG,
      outcome({ result: result({ publication: { kind: "unnecessary" }, unchanged: 42 }) })
    );
    const previewed: IJobNotice = describePatchOutcome(
      CONFIG,
      outcome({ result: result({ modified: [change("configs\\system.ltx")] }) })
    );

    expect(agreed.title).toBe("Nothing to patch");
    expect(previewed.title).toBe("Compared archives");
    expect(previewed.details).toContain("1 entry(s) would be carried");
  });

  it("reports a failure with the reason rather than the destination", () => {
    const notification: IJobNotice = describePatchOutcome(
      CONFIG,
      outcome({ error: new Error("base root holds no entry to compare") })
    );

    expect(notification.severity).toBe(ENotificationSeverity.ERROR);
    expect(notification.details).toContain("base root holds no entry to compare");
  });

  it("reports a cancelled run as leaving the destination alone", () => {
    const notification: IJobNotice = describePatchOutcome(
      CONFIG,
      outcome({ result: result({ outcome: "cancelled" }) })
    );

    expect(notification.severity).toBe(ENotificationSeverity.INFO);
    expect(notification.details).toContain("left as it was found");
  });
});
