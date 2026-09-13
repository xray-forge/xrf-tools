import { beforeEach, describe, expect, it } from "@jest/globals";

import { ConfigsProjectDescriptor } from "@/core/ipc/types/xrf-app";
import { LtxResolvedIndex, LtxResolvedSection } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

function mockDeferredSections() {
  let resolve: (sections: Array<LtxResolvedSection>) => void = noop;
  const promise = new Promise<Array<LtxResolvedSection>>((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

function getEntryOf(name: string, origin: string): LtxResolvedIndex["sections"][number] {
  return { fieldCount: 1, name, origin, parents: [] };
}

function getIndexOf(sections: LtxResolvedIndex["sections"]): LtxResolvedIndex {
  return { dialect: "ltx", diagnostics: [], entry: "system.ltx", sections };
}

function getSectionOf(name: string): LtxResolvedSection {
  return { entry: "system.ltx", fields: [], name, origin: "items\\w_base.ltx", parents: [] };
}

function mockOpenedService(): ConfigsResolvedService {
  const { service, container } = mockInjectedService(ConfigsResolvedService, [ConfigsProjectService]);
  const project = container.get(ConfigsProjectService);

  project.project = project.project.asReady({ sessionId: "session-1" } as ConfigsProjectDescriptor);

  return service;
}

describe("ConfigsResolvedService", () => {
  beforeEach(() => setMockInvokeResponses({}));

  it("indexes an entry point once and answers the second ask from what it holds", async () => {
    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([getEntryOf("wpn_base", "items\\w_base.ltx")]),
    });

    const service: ConfigsResolvedService = mockOpenedService();

    await service.open("system.ltx");
    await service.open("system.ltx");

    expect(mockInvoke.mock.calls.filter(([name]) => name === "plugin:configs|list_resolved_sections")).toHaveLength(1);
    expect(service.index.value?.sections).toHaveLength(1);
  });

  it("asks only for the sections it does not already hold", async () => {
    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([getEntryOf("a", "one.ltx"), getEntryOf("b", "one.ltx")]),
      "plugin:configs|read_resolved_sections": [getSectionOf("a")],
    });

    const service: ConfigsResolvedService = mockOpenedService();

    await service.open("system.ltx");
    await service.request(["a"]);
    await service.request(["a"]);

    // A page turn that scrolls back over what it already read must not re-ask: the whole point of paging by name is
    // that a name answered once stays answered.
    const pages = mockInvoke.mock.calls.filter(([name]) => name === "plugin:configs|read_resolved_sections");

    expect(pages).toHaveLength(1);
    expect(service.sections.get("a")).toBeDefined();
  });

  it("drops a page that lands after the entry point it was asked for was replaced", async () => {
    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([getEntryOf("a", "one.ltx")]),
      "plugin:configs|read_resolved_sections": [getSectionOf("a")],
    });

    const service: ConfigsResolvedService = mockOpenedService();

    await service.open("system.ltx");

    const page: Promise<void> = service.request(["a"]);

    // Reopening while the page is in flight: the answer describes a document nobody is looking at any more.
    await service.open("other.ltx");
    await page;

    expect(service.sections.has("a")).toBe(false);
  });

  it("narrows to what one config declared, and widens back", async () => {
    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([
        getEntryOf("wpn_base", "items\\w_base.ltx"),
        getEntryOf("system", "system.ltx"),
      ]),
    });

    const service: ConfigsResolvedService = mockOpenedService();

    await service.open("system.ltx");

    expect(service.visibleSections).toHaveLength(2);

    service.narrowTo("items\\w_base.ltx");

    // What a person opening an included config wants: what that file's sections came to, not where they sit among the
    // thousands its entry point resolves.
    expect(service.visibleSections.map((section) => section.name)).toEqual(["wpn_base"]);

    service.narrowTo(null);

    expect(service.visibleSections).toHaveLength(2);
  });

  it("keeps a cleared index empty when its read finishes", async () => {
    let resolve: (index: LtxResolvedIndex) => void = noop;
    const response = new Promise<LtxResolvedIndex>((settle) => {
      resolve = settle;
    });

    setMockInvokeResponses({ "plugin:configs|list_resolved_sections": () => response });

    const service = mockOpenedService();
    const opening = service.open("system.ltx");

    service.clear();
    resolve(getIndexOf([getEntryOf("a", "one.ltx")]));
    await opening;

    expect(service.entry).toBeNull();
    expect(service.index.isIdle).toBe(true);
    expect(service.index.value).toBeNull();
  });

  it("drops a page that lands after clear", async () => {
    const response = mockDeferredSections();

    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([getEntryOf("a", "one.ltx")]),
      "plugin:configs|read_resolved_sections": () => response.promise,
    });

    const service = mockOpenedService();

    await service.open("system.ltx");

    const page = service.request(["a"]);

    service.clear();

    const revision = service.revision;

    response.resolve([getSectionOf("a")]);
    await page;

    expect(service.sections.size).toBe(0);
    expect(service.revision).toBe(revision);
  });

  it("drops a page from an earlier opening of the same entry", async () => {
    const response = mockDeferredSections();

    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([getEntryOf("a", "one.ltx")]),
      "plugin:configs|read_resolved_sections": () => response.promise,
    });

    const service = mockOpenedService();

    await service.open("system.ltx");

    const page = service.request(["a"]);

    service.clear();
    await service.open("system.ltx");

    const revision = service.revision;

    response.resolve([getSectionOf("a")]);
    await page;

    expect(service.sections.size).toBe(0);
    expect(service.revision).toBe(revision);
    expect(service.index.isReady).toBe(true);
  });

  it("keeps newer pending sections reserved when an abandoned page finishes", async () => {
    const older = mockDeferredSections();
    const newer = mockDeferredSections();

    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([getEntryOf("a", "one.ltx")]),
      "plugin:configs|read_resolved_sections": () => older.promise,
    });

    const service = mockOpenedService();

    await service.open("system.ltx");

    const first = service.request(["a"]);

    service.clear();
    await service.open("system.ltx");

    setMockInvokeResponses({ "plugin:configs|read_resolved_sections": () => newer.promise });

    const second = service.request(["a"]);

    older.resolve([]);
    await first;

    const repeated = service.request(["a"]);

    newer.resolve([getSectionOf("a")]);
    await Promise.all([second, repeated]);

    expect(mockInvoke.mock.calls.filter(([name]) => name === "plugin:configs|read_resolved_sections")).toHaveLength(2);
    expect(service.sections.get("a")).toEqual(getSectionOf("a"));
  });

  it("accepts independent pages of the current resolution in either completion order", async () => {
    const first = mockDeferredSections();
    const second = mockDeferredSections();

    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": getIndexOf([getEntryOf("a", "one.ltx"), getEntryOf("b", "one.ltx")]),
      "plugin:configs|read_resolved_sections": () => first.promise,
    });

    const service = mockOpenedService();

    await service.open("system.ltx");

    const firstPage = service.request(["a"]);

    setMockInvokeResponses({ "plugin:configs|read_resolved_sections": () => second.promise });

    const secondPage = service.request(["b"]);

    second.resolve([getSectionOf("b")]);
    await secondPage;
    first.resolve([getSectionOf("a")]);
    await firstPage;

    expect(service.sections.get("a")).toEqual(getSectionOf("a"));
    expect(service.sections.get("b")).toEqual(getSectionOf("b"));
  });
});
