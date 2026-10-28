import { describe, expect, it, jest } from "@jest/globals";

import { mockInjectedService } from "@/fixtures/utils/container";
import { LEVEL_VIEWER_TOOL } from "@/tools/level-viewer";
import { VISUALS_EXPLORER_TOOL } from "@/tools/visuals-explorer";

import { ShellToolService } from "./shell-tool.service";

describe("ShellToolService", () => {
  it("follows the route's tool and leaves it through the route", () => {
    const { service } = mockInjectedService(ShellToolService);
    const leave = jest.fn();

    service.follow(LEVEL_VIEWER_TOOL, false, leave);

    expect(service.isToolOpen).toBe(true);
    expect(service.isLauncher).toBe(false);

    service.close();

    expect(leave).toHaveBeenCalledTimes(1);
  });

  it("leaves nothing from the launcher", () => {
    const { service } = mockInjectedService(ShellToolService);
    const leave = jest.fn();

    service.follow(null, true, leave);
    service.close();

    expect(service.isToolOpen).toBe(false);
    expect(service.isLauncher).toBe(true);
    expect(leave).not.toHaveBeenCalled();
  });

  it("keeps a newer route when an older frame releases its own", () => {
    const { service } = mockInjectedService(ShellToolService);
    const older = jest.fn();
    const newer = jest.fn();

    service.follow(LEVEL_VIEWER_TOOL, false, older);
    service.follow(VISUALS_EXPLORER_TOOL, false, newer);
    service.release(older);
    service.close();

    expect(service.tool).toBe(VISUALS_EXPLORER_TOOL);
    expect(newer).toHaveBeenCalledTimes(1);

    service.release(newer);

    expect(service.isToolOpen).toBe(false);
  });
});
