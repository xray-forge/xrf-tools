import { describe, expect, it } from "@jest/globals";

import { JobProgress, ProgressLevel, ProgressUnit } from "@/core/bindings/types/xrf-job";
import { describeActiveProgress, formatProgressCounts, toProgressPercent } from "@/core/jobs/lib/progress-format";
import { Nullable } from "@/lib/types/general";

function level(
  id: string,
  completed: number,
  total: Nullable<number>,
  unit: ProgressUnit = "items",
  label: Nullable<string> = null
): ProgressLevel {
  return { id, label, completed, total, unit };
}

function progress(levels: Array<ProgressLevel>, detail: Nullable<string> = null): JobProgress {
  return { levels, duration: 1000, detail };
}

describe("formatProgressCounts", () => {
  it("groups a count of things", () => {
    expect(formatProgressCounts(level("write", 45000, 100000))).toBe(
      `${(45000).toLocaleString()} / ${(100000).toLocaleString()}`
    );
  });

  it("renders a count of bytes as a size", () => {
    // The same two numbers read differently depending on the unit, which is why the unit travels with the level rather
    // than being inferred by whoever draws it.
    expect(formatProgressCounts(level("write", 1024, 2048, "bytes"))).toBe("1 KB / 2 KB");
  });

  it("reports a bare count where there is no total", () => {
    expect(formatProgressCounts(level("collect", 12, null))).toBe("12");
  });
});

describe("toProgressPercent", () => {
  it("measures how far a countable level has got", () => {
    expect(toProgressPercent(level("write", 25, 100))).toBe(25);
  });

  it("refuses to measure a level that does not know its size", () => {
    // A pack walking its source has no total until the walk ends. Inventing one gives the user a number to act on.
    expect(toProgressPercent(level("collect", 12, null))).toBeNull();
  });

  it("treats a zero total as unknown rather than finished", () => {
    // Dividing by it would report either nothing or everything, and both are claims the level has not made.
    expect(toProgressPercent(level("write", 0, 0))).toBeNull();
  });

  it("clamps a level that overshot its total", () => {
    // An unpack counts directory rows it never wrote, and a total measured before the work can be an entry out. A bar
    // past its end reads as a broken tool rather than an imprecise count.
    expect(toProgressPercent(level("write", 120, 100))).toBe(100);
  });
});

describe("describeActiveProgress", () => {
  it("says nothing when there is no snapshot yet", () => {
    expect(describeActiveProgress(null)).toBe("");
  });

  it("names the levels too deep to draw", () => {
    expect(
      describeActiveProgress(
        progress([level("verify", 2, 7), level("textures", 1, 3), level("assets", 400, 40000)])
      )
    ).toBe("assets");
  });

  it("prefers a deeper level to the job's own detail", () => {
    // The deeper level is the more specific answer. `detail` names whichever file a worker holds, which is a weaker
    // description of where the run actually is.
    expect(
      describeActiveProgress(
        progress([level("verify", 2, 7), level("textures", 1, 3), level("assets", 400, 40000)], "act_hood.dds")
      )
    ).toBe("assets");
  });

  it("falls back to the detail a sequential run reports", () => {
    expect(describeActiveProgress(progress([level("write", 3, 9)], "meshes/actor.ogf"))).toBe("meshes/actor.ogf");
  });

  it("says nothing when a parallel run declines to name an entry", () => {
    // A parallel operation deliberately leaves `detail` empty: naming one arbitrary worker's entry reads as thrashing.
    expect(describeActiveProgress(progress([level("write", 3, 9)]))).toBe("");
  });
});
