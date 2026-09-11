import { beforeEach, describe, expect, it } from "@jest/globals";

import { ExportsService } from "@/applications/exports-explorer/services/exports/exports.service";
import { ExportsProject } from "@/core/bindings/types/xrf-export";
import { mockDocumentResponse } from "@/fixtures/mocks/document.mocks";
import { mockExportsProject } from "@/fixtures/mocks/project.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const PROJECT: ExportsProject = mockExportsProject();

describe("ExportsService", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockDocumentResponse(null),
      ["plugin:exports|open_project"]: mockDocumentResponse(PROJECT),
      ["plugin:exports|close_project"]: undefined,
    });
  });

  it("restores an existing backend session", async () => {
    setMockInvokeResponses({ ["plugin:exports|get_project"]: mockDocumentResponse(PROJECT) });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.project.value).toEqual({ ...PROJECT, sessionId: expect.any(String) });
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
      ["plugin:exports|get_project"]: mockDocumentResponse(() => {
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
    expect(service.project.value).toEqual({ ...PROJECT, sessionId: expect.any(String) });
  });

  it("keeps the last successful project when refresh fails", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockDocumentResponse(PROJECT),
      ["plugin:exports|open_project"]: mockDocumentResponse(() => {
        throw new Error("parse failed");
      }),
    });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();
    await service.refreshExportsProject();

    expect(service.project.value).toEqual({ ...PROJECT, sessionId: expect.any(String) });
    expect(service.project.isLoading).toBe(false);
    expect(service.project.error).toEqual(new Error("parse failed"));
  });

  it("keeps the project and rejects when close fails", async () => {
    setMockInvokeResponses({
      ["plugin:exports|get_project"]: mockDocumentResponse(PROJECT),
      ["plugin:exports|close_project"]: () => {
        throw new Error("project is busy");
      },
    });

    const service = mockInjectedService(ExportsService).service;

    await service.onProvision();

    await expect(service.closeExportsProject()).rejects.toThrow("project is busy");
    expect(service.project.value).toEqual({ ...PROJECT, sessionId: expect.any(String) });
    expect(service.project.isLoading).toBe(false);
  });
});
