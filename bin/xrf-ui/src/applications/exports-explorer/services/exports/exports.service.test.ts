import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { reaction } from "@wirestate/mobx";

import { ExportsService } from "@/applications/exports-explorer/services/exports/exports.service";
import { ExportsProject } from "@/core/ipc/types/xrf-export";
import { mockExportsProject } from "@/fixtures/mocks/project.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const PROJECT: ExportsProject = mockExportsProject();

describe("ExportsService", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockSessionResponse(null),
      ["plugin:exports|open_project"]: mockSessionResponse(PROJECT),
      ["plugin:exports|close_project"]: undefined,
    });
  });

  it("restores an existing backend session", async () => {
    setMockInvokeResponses({ ["plugin:exports|get_project"]: mockSessionResponse(PROJECT) });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.project.value).toEqual(PROJECT);
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:exports|open_project", expect.anything());
  });

  it("does not open a project when no retained session exists", async () => {
    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.project.value).toBeNull();
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:exports|get_project");
  });

  it("recovers from a failed session lookup", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockSessionResponse(() => {
        throw new Error("backend unavailable");
      }),
    });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.project.isLoading).toBe(false);
    expect(service.project.error).toEqual(new Error("backend unavailable"));
  });

  it("opens only the explicitly provided project", async () => {
    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();
    await service.openExportsProject("C:\\chosen\\xrf");

    expect(mockInvoke).toHaveBeenCalledWith("plugin:exports|open_project", {
      sessionId: expect.any(String),
      projectPath: "C:\\chosen\\xrf",
    });
    expect(service.project.value).toEqual(PROJECT);
  });

  it("retains projected data and reports refresh status when a replacement fails", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockSessionResponse(PROJECT),
      ["plugin:exports|open_project"]: mockSessionResponse(() => {
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

  it("keeps the project and rejects when close fails", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockSessionResponse(PROJECT),
      ["plugin:exports|close_project"]: () => {
        throw new Error("project is busy");
      },
    });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    await expect(service.closeExportsProject()).rejects.toThrow("project is busy");
    expect(service.project.value).toEqual(PROJECT);
    expect(service.project.isLoading).toBe(false);
  });
});
