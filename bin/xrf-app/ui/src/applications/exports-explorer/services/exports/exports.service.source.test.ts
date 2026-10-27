import { beforeEach, describe, expect, it } from "@jest/globals";
import { EExportsCommand } from "@xrf/ipc/commands/exports";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ExportSourceContent } from "@xrf/ipc/types/xrf-export";

import { mockExportsProject } from "@/applications/exports-explorer/fixtures/exports.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ExportsService } from "./exports.service";

const SOURCE: ExportSourceContent = {
  name: "xr_effects.play",
  path: "effects/sound.ts",
  line: 18,
  endLine: 21,
  content: 'extern("xr_effects.play", (): void => {});',
};

async function mockRestoredService(): Promise<ExportsService> {
  const service: ExportsService = mockInjectedService(ExportsService).service;

  await service.onProvision();

  return service;
}

describe("ExportsService export source", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EExportsCommand.GET_SOURCE]: SOURCE,
      [EExportsCommand.RESTORE_PROJECT]: mockSessionSnapshot(mockExportsProject()),
    });
  });

  it("reads the source of one declaration by name", async () => {
    const service: ExportsService = await mockRestoredService();

    await expect(service.readExportSource("xr_effects.play")).resolves.toEqual(SOURCE);

    expect(mockInvoke).toHaveBeenCalledWith(EExportsCommand.GET_SOURCE, {
      sessionId: expect.any(String),
      name: "xr_effects.play",
    });
  });

  it("propagates a failed read to its caller", async () => {
    // Reporting is the view's job here, so the service must not swallow this into a null result.
    const service: ExportsService = await mockRestoredService();

    mockIpcCommands({
      [EExportsCommand.GET_SOURCE]: () => {
        throw new Error("declaration file is gone");
      },
    });

    await expect(service.readExportSource("xr_effects.play")).rejects.toThrow("declaration file is gone");
  });

  it("holds no source state of its own", () => {
    // The body belongs to whatever is on screen; keeping it here would make the service arbitrate
    // between in-flight reads that the viewing effect already knows how to abandon.
    expect(mockInjectedService(ExportsService).service).not.toHaveProperty("source");
  });
});
