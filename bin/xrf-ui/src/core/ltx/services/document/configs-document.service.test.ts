import { describe, expect, it } from "@jest/globals";

import { ConfigsDocument, ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

function mockDocumentOf(path: string): ConfigsDocument {
  return {
    findings: [],
    structure: { entryPoints: [], includes: [], parseError: null, path, rootEntries: [], sections: [] },
    text: { isNormalized: true, lines: [], path },
  };
}

function mockOpenedService(): ConfigsDocumentService {
  const { service, container } = mockInjectedService(ConfigsDocumentService, [ConfigsProjectService]);
  const project = container.get(ConfigsProjectService);

  project.project = project.project.asReady({ sessionId: "session-1" } as ConfigsProjectDescriptor);

  return service;
}

describe("ConfigsDocumentService", () => {
  it("keeps a cleared document empty when its read finishes", async () => {
    let resolve: (document: ConfigsDocument) => void = noop;
    const response = new Promise<ConfigsDocument>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:configs|read_document": () => response });

    const service = mockOpenedService();
    const reading = service.select("system.ltx");

    service.clear();
    resolve(mockDocumentOf("system.ltx"));
    await reading;

    expect(service.selected).toBeNull();
    expect(service.document.isIdle).toBe(true);
    expect(service.document.value).toBeNull();
  });

  it("keeps a cleared document idle when its read fails", async () => {
    let reject: (error: Error) => void = noop;
    const response = new Promise<ConfigsDocument>((_resolve, fail) => {
      reject = fail;
    });

    setMockInvokeResponses({ "plugin:configs|read_document": () => response });

    const service = mockOpenedService();
    const reading = service.select("system.ltx");

    service.clear();
    reject(new Error("Project closed"));
    await reading;

    expect(service.document.isIdle).toBe(true);
    expect(service.document.error).toBeNull();
  });

  it("keeps the current document visible until its replacement arrives", async () => {
    const previous = mockDocumentOf("previous.ltx");
    const replacement = mockDocumentOf("replacement.ltx");
    let resolve: (document: ConfigsDocument) => void = noop;
    const response = new Promise<ConfigsDocument>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:configs|read_document": previous });

    const service = mockOpenedService();

    await service.select("previous.ltx");
    setMockInvokeResponses({ "plugin:configs|read_document": () => response });

    const reading = service.select("replacement.ltx");

    expect(service.document.isLoading).toBe(true);
    expect(service.document.value).toEqual(previous);

    resolve(replacement);
    await reading;

    expect(service.document.isReady).toBe(true);
    expect(service.document.value).toEqual(replacement);
  });
});
