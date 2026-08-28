import { beforeEach, describe, expect, it } from "@jest/globals";

import { EWorkspacePath, IWorkspacePathDescriptor, WORKSPACE_PATHS } from "@/core/settings/lib/workspace-path";
import { PathsService } from "@/core/settings/services/paths/paths.service";
import { mockInjectedService } from "@/fixtures/utils/container";

describe("PathsService", () => {
  const GAMEDATA: string = "D:\\mods\\my-mod\\gamedata";

  beforeEach(() => {
    window.localStorage.clear();
  });

  it("describes every path exactly once, in the order the settings screen lists them", () => {
    // The table is the only place a path is declared, and an ordered list cannot prove on its own that it covers the
    // whole vocabulary. Asserting it here keeps a forgotten entry from silently losing a setting.
    expect(WORKSPACE_PATHS.map((it: IWorkspacePathDescriptor) => it.id)).toEqual(Object.values(EWorkspacePath));
  });

  it("starts with nothing configured", () => {
    const { service } = mockInjectedService(PathsService);

    for (const { id } of WORKSPACE_PATHS) {
      expect(service.getPath(id)).toBeNull();
    }
  });

  it("reads every remembered path as it is constructed", () => {
    // Read rather than restored, so a tool mounting before the settings screen ever opens still derives from it.
    window.localStorage.setItem("xrf-gamedata-path", GAMEDATA);
    window.localStorage.setItem("xrf-output-path", "D:\\work\\out");

    const { service } = mockInjectedService(PathsService);

    expect(service.getPath(EWorkspacePath.GAMEDATA)).toBe(GAMEDATA);
    expect(service.getPath(EWorkspacePath.OUTPUT)).toBe("D:\\work\\out");
    expect(service.getPath(EWorkspacePath.CONFIGS)).toBeNull();
  });

  it("keeps a stored path that is no longer there, rather than dropping it silently", () => {
    // Whether a path still exists is reported where it is shown. Forgetting it here would look like the setting was
    // lost, and would quietly stop every tool deriving from it.
    window.localStorage.setItem("xrf-gamedata-path", "Z:\\unplugged\\gamedata");

    const { service } = mockInjectedService(PathsService);

    expect(service.getPath(EWorkspacePath.GAMEDATA)).toBe("Z:\\unplugged\\gamedata");
  });

  it("persists each path under its own key", () => {
    const { service } = mockInjectedService(PathsService);

    for (const { id, storageKey } of WORKSPACE_PATHS) {
      service.setPath(id, `C:\\${id}`);

      expect(service.getPath(id)).toBe(`C:\\${id}`);
      expect(window.localStorage.getItem(storageKey)).toBe(`C:\\${id}`);
    }
  });

  it("forgets a cleared path and leaves the others alone", () => {
    const { service } = mockInjectedService(PathsService);

    service.setPath(EWorkspacePath.GAMEDATA, GAMEDATA);
    service.setPath(EWorkspacePath.CONFIGS, "C:\\src\\engine\\configs");

    service.setPath(EWorkspacePath.CONFIGS, null);

    expect(service.getPath(EWorkspacePath.CONFIGS)).toBeNull();
    expect(window.localStorage.getItem("xrf-configs-path")).toBeNull();
    expect(service.getPath(EWorkspacePath.GAMEDATA)).toBe(GAMEDATA);
  });

  it("publishes a new record, so anything deriving from it sees the change", () => {
    const { service } = mockInjectedService(PathsService);

    const before = service.paths;

    service.setPath(EWorkspacePath.GAMEDATA, GAMEDATA);

    expect(service.paths).not.toBe(before);
    expect(before[EWorkspacePath.GAMEDATA]).toBeNull();
  });
});
