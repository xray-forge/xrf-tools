import { beforeEach, describe, expect, it } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { LevelEntry } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { LevelListService } from "@/core/level/services/level-list.service";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const ROOTS: XrayRoots = createRoots(["C:\\game\\db"]);

function mockEntry(name: string, hasGeometry: boolean = true): LevelEntry {
  return { hasGeometry, logicalPath: `levels\\${name}`, name };
}

describe("LevelListService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(LevelListService);

    expect(isObservableProp(service, "levels")).toBe(true);
    expect(isComputedProp(service, "drawable")).toBe(true);
  });

  it("lists what the roots hold", async () => {
    const { service } = mockInjectedService(LevelListService);

    setMockInvokeResponses({ ["plugin:levels|list_levels"]: [mockEntry("zaton"), mockEntry("jupiter")] });

    await service.list(ROOTS);

    expect(service.levels.value?.map((it: LevelEntry) => it.name)).toEqual(["zaton", "jupiter"]);
  });

  // A level with no render geometry opens and draws nothing, so a picker offering it only wastes the click.
  it("separates the levels there is something to draw for", async () => {
    const { service } = mockInjectedService(LevelListService);

    setMockInvokeResponses({ ["plugin:levels|list_levels"]: [mockEntry("zaton"), mockEntry("lmaps", false)] });

    await service.list(ROOTS);

    expect(service.drawable.map((it: LevelEntry) => it.name)).toEqual(["zaton"]);
  });

  // A listing is only about the roots it was made from, so a picker pointed somewhere else has it forget.
  it("forgets a listing when asked", async () => {
    const { service } = mockInjectedService(LevelListService);

    setMockInvokeResponses({ ["plugin:levels|list_levels"]: [mockEntry("zaton")] });

    await service.list(ROOTS);
    service.reset();

    expect(service.levels.value).toBeNull();
    expect(service.drawable).toEqual([]);
  });

  it("records a failure as state rather than throwing it at the caller", async () => {
    const { service } = mockInjectedService(LevelListService);

    setMockInvokeResponses({
      ["plugin:levels|list_levels"]: () => {
        throw new Error("no roots are mounted");
      },
    });

    await service.list(ROOTS);

    expect(service.levels.value).toBeNull();
    expect(service.levels.error?.message).toBe("no roots are mounted");
  });
});
