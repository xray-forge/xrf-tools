import { describe, expect, it, jest } from "@jest/globals";

import { EToolId } from "@/core/routing/tool";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ShellToolService } from "./shell-tool.service";

describe("ShellToolService", () => {
  it("follows the route's tool and leaves it through the route", () => {
    const { service } = mockInjectedService(ShellToolService);
    const leave = jest.fn();

    service.follow(EToolId.LEVEL_VIEWER, leave);

    expect(service.isToolOpen).toBe(true);

    service.close();

    expect(leave).toHaveBeenCalledTimes(1);
  });

  it("leaves nothing from the launcher", () => {
    const { service } = mockInjectedService(ShellToolService);
    const leave = jest.fn();

    service.follow(null, leave);
    service.close();

    expect(service.isToolOpen).toBe(false);
    expect(leave).not.toHaveBeenCalled();
  });

  it("keeps a newer route when an older frame releases its own", () => {
    const { service } = mockInjectedService(ShellToolService);
    const older = jest.fn();
    const newer = jest.fn();

    service.follow(EToolId.LEVEL_VIEWER, older);
    service.follow(EToolId.VISUALS_EXPLORER, newer);
    service.release(older);
    service.close();

    expect(service.tool).toBe(EToolId.VISUALS_EXPLORER);
    expect(newer).toHaveBeenCalledTimes(1);

    service.release(newer);

    expect(service.isToolOpen).toBe(false);
  });
});
