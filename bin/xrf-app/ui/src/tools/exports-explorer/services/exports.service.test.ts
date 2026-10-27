import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { reaction } from "@wirestate/mobx";
import { EExportsCommand } from "@xrf/ipc/commands/exports";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { ExportsProject } from "@xrf/ipc/types/xrf-export";

import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { mockExportsProject } from "@/tools/exports-explorer/fixtures/exports.mocks";

import { ExportsService } from "./exports.service";

const PROJECT: ExportsProject = mockExportsProject();

describe("ExportsService", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionResponse(null),
      [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(PROJECT),
      [EExportsCommand.CLOSE_PROJECT]: undefined,
    });
  });

  it("restores an existing backend session", async () => {
    mockIpcCommands({ [EExportsCommand.RESTORE_PROJECT]: mockSessionResponse(PROJECT) });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    expect(service.project.isRestored).toBe(true);
    expect(service.project.value).toEqual(PROJECT);
    expect(mockInvoke).not.toHaveBeenCalledWith(EExportsCommand.OPEN_PROJECT, expect.anything());
  });

  it("does not open a project when no retained session exists", async () => {
    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    expect(service.project.isRestored).toBe(true);
    expect(service.project.value).toBeNull();
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    // The restore names the tool's scope, which the backend lists its openings by.
    expect(mockInvoke).toHaveBeenCalledWith(EExportsCommand.RESTORE_PROJECT, {
      tool: "shell",
    });
  });

  it("recovers from a failed session lookup", async () => {
    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionResponse(() => {
        throw new Error("backend unavailable");
      }),
    });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    expect(service.project.isRestored).toBe(true);
    expect(service.project.isLoading).toBe(false);
    expect(service.project.error).toEqual(new Error("backend unavailable"));
  });

  it("opens only the explicitly provided project", async () => {
    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();
    await service.openExportsProject("C:\\chosen\\xrf");

    expect(mockInvoke).toHaveBeenCalledWith(EExportsCommand.OPEN_PROJECT, {
      sessionId: expect.any(String),
      projectPath: "C:\\chosen\\xrf",
    });
    expect(service.project.value).toEqual(PROJECT);
  });

  it("retains projected data and reports refresh status when a replacement fails", async () => {
    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionResponse(PROJECT),
      [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(() => {
        throw new Error("parse failed");
      }),
    });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    const projectChanged = jest.fn();
    const loadingChanges: Array<boolean> = [];
    const stopProject = reaction(() => service.project.value, projectChanged);
    const stopLoading = reaction(
      () => service.project.isLoading,
      (loading) => loadingChanges.push(loading)
    );

    try {
      await service.refreshExportsProject();

      expect(service.project.value).toBe(PROJECT);
      expect(projectChanged).not.toHaveBeenCalled();
      expect(loadingChanges).toEqual([true, false]);
      expect(service.project.error).toEqual(new Error("parse failed"));
    } finally {
      stopProject();
      stopLoading();
    }
  });

  it("forgets the manifest of a project it closes", async () => {
    const service = mockInjectedService(ExportsService).service;

    await service.openExportsProject("C:\\chosen\\xrf");
    await service.exportManifest("C:\\out\\extern.json");

    expect(service.manifest.value).toBe("C:\\out\\extern.json");

    await service.closeExportsProject();

    expect(service.project.value).toBeNull();
    expect(service.manifest.state.isIdle).toBe(true);
    expect(service.manifest.value).toBeNull();
  });

  it("keeps the project when close fails", async () => {
    mockIpcCommands({
      [EExportsCommand.RESTORE_PROJECT]: mockSessionResponse(PROJECT),
      [EExportsCommand.CLOSE_PROJECT]: () => {
        throw new Error("project is busy");
      },
    });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    await service.closeExportsProject();

    expect(service.project.value).toEqual(PROJECT);
    expect(service.project.isLoading).toBe(false);
  });
});
