import { describe, expect, it } from "@jest/globals";

import { sumWebviewMemory, toMemoryFigures } from "@/core/diagnostics/lib/memory-figures";
import { EWebviewProcessKind } from "@/core/ipc/types/xrf-app";

describe("toMemoryFigures", () => {
  it("counts the private working set as in use", () => {
    expect(toMemoryFigures({ committed: 9, workingSet: 5, privateWorkingSet: 3 })).toEqual({ inUse: 3, committed: 9 });
  });

  it("counts the whole working set as in use where the host reports no private one", () => {
    expect(toMemoryFigures({ committed: 9, workingSet: 5, privateWorkingSet: null })).toEqual({
      inUse: 5,
      committed: 9,
    });
  });
});

describe("sumWebviewMemory", () => {
  it("sums every process's figures, and nothing where none is reported", () => {
    expect(sumWebviewMemory([])).toEqual({ inUse: 0, committed: 0 });
    expect(
      sumWebviewMemory([
        { kind: EWebviewProcessKind.RENDERER, pid: 1, memory: { committed: 9, workingSet: 5, privateWorkingSet: 3 } },
        { kind: EWebviewProcessKind.GPU, pid: 2, memory: { committed: 40, workingSet: 2, privateWorkingSet: 1 } },
      ])
    ).toEqual({ inUse: 4, committed: 49 });
  });
});
