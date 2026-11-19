import { describe, expect, it, jest } from "@jest/globals";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { HostInfo } from "@xrf/ipc/types/xrf-app-system";
import { BuildInfo } from "@xrf/ipc/types/xrf-build-info";
import { RenderLibraries } from "@xrf/ipc/types/xrf-renderer";

import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";
import { mockInjectedService } from "@/fixtures/utils/container";

// Only identity matters here: the sections' own tests describe what each holds.
const BUILD: BuildInfo = { version: "1.0.0" } as unknown as BuildInfo;
const HOST: HostInfo = { os: "windows" } as unknown as HostInfo;
const LIBRARIES: RenderLibraries = { dlss: null } as unknown as RenderLibraries;

function countCalls(command: string): number {
  return mockInvoke.mock.calls.filter(([called]) => called === command).length;
}

describe("SettingsAboutService", () => {
  it("reads what the section reports once and keeps it", async () => {
    mockIpcCommands({
      [ESystemCommand.GET_BUILD_INFO]: BUILD,
      [ESystemCommand.GET_HOST_INFO]: HOST,
      [ESystemCommand.GET_ENVIRONMENT_FLAGS]: [],
      [ERenderCommand.DESCRIBE_LIBRARIES]: LIBRARIES,
    });

    const { service } = mockInjectedService(SettingsAboutService);

    await service.read();
    await service.read();

    expect(service.build.value).toBe(BUILD);
    expect(service.host.value).toBe(HOST);
    expect(service.variables.value).toEqual([]);
    expect(service.libraries.value).toBe(LIBRARIES);
    expect(countCalls(ESystemCommand.GET_BUILD_INFO)).toBe(1);
  });

  it("reads one that failed again the next time, and the others go on without it", async () => {
    const build = jest.fn((): Promise<BuildInfo> => Promise.reject(new Error("no build info")));

    mockIpcCommands({
      [ESystemCommand.GET_BUILD_INFO]: build,
      [ESystemCommand.GET_HOST_INFO]: HOST,
      [ESystemCommand.GET_ENVIRONMENT_FLAGS]: [],
      [ERenderCommand.DESCRIBE_LIBRARIES]: LIBRARIES,
    });

    const { service } = mockInjectedService(SettingsAboutService);

    await service.read();

    expect(service.build.error?.message).toContain("no build info");
    expect(service.host.value).toBe(HOST);

    await service.read();

    expect(build).toHaveBeenCalledTimes(2);
    expect(countCalls(ESystemCommand.GET_HOST_INFO)).toBe(1);
  });
});
