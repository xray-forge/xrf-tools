import { beforeEach, describe, expect, it } from "@jest/globals";
import { EConfigsCommand } from "@xrf/ipc/commands/configs";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { LtxResolvedIndex, LtxResolvedSection } from "@xrf/ipc/types/xrf-ltx-inspect";

import { mockRestoredConfigsProject } from "@/core/ltx/fixtures/configs-project";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { mockSessionRead } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

function getEntryOf(name: string, origin: string): LtxResolvedIndex["sections"][number] {
  return { fieldCount: 1, name, origin, parents: [] };
}

function getIndexOf(sections: LtxResolvedIndex["sections"]): LtxResolvedIndex {
  return { dialect: "ltx", diagnostics: [], entry: "system.ltx", sections };
}

function getSectionOf(name: string): LtxResolvedSection {
  return { entry: "system.ltx", fields: [], name, origin: "items\\w_base.ltx", parents: [] };
}

async function mockOpenedService(): Promise<ConfigsResolvedService> {
  const { service, container } = mockInjectedService(ConfigsResolvedService, [ConfigsProjectService]);

  await mockRestoredConfigsProject(container.get(ConfigsProjectService));

  return service;
}

describe("ConfigsResolvedService", () => {
  beforeEach(() => mockIpcCommands({}));

  it("indexes an entry point once and answers the second ask from what it holds", async () => {
    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(
        getIndexOf([getEntryOf("wpn_base", "items\\w_base.ltx")])
      ),
    });

    const service: ConfigsResolvedService = await mockOpenedService();

    await service.open("system.ltx");
    await service.open("system.ltx");

    expect(mockInvoke.mock.calls.filter(([name]) => name === EConfigsCommand.LIST_RESOLVED_SECTIONS)).toHaveLength(1);
    expect(service.index.value?.sections).toHaveLength(1);
  });

  it("asks only for the sections it does not already hold", async () => {
    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(
        getIndexOf([getEntryOf("a", "one.ltx"), getEntryOf("b", "one.ltx")])
      ),
      [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead([getSectionOf("a")]),
    });

    const service: ConfigsResolvedService = await mockOpenedService();

    await service.open("system.ltx");
    await service.request(["a"]);
    await service.request(["a"]);

    // A page turn that scrolls back over what it already read must not re-ask: the whole point of paging by name is
    // that a name answered once stays answered.
    const pages = mockInvoke.mock.calls.filter(([name]) => name === EConfigsCommand.READ_RESOLVED_SECTIONS);

    expect(pages).toHaveLength(1);
    expect(service.sections.get("a")).toBeDefined();
  });

  it("drops a page that lands after the entry point it was asked for was replaced", async () => {
    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(getIndexOf([getEntryOf("a", "one.ltx")])),
      [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead([getSectionOf("a")]),
    });

    const service: ConfigsResolvedService = await mockOpenedService();

    await service.open("system.ltx");

    const page: Promise<void> = service.request(["a"]);

    // Reopening while the page is in flight: the answer describes a document nobody is looking at any more.
    await service.open("other.ltx");
    await page;

    expect(service.sections.has("a")).toBe(false);
  });

  it("narrows to what one config declared, and widens back", async () => {
    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(
        getIndexOf([getEntryOf("wpn_base", "items\\w_base.ltx"), getEntryOf("system", "system.ltx")])
      ),
    });

    const service: ConfigsResolvedService = await mockOpenedService();

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
    const response = deferred<LtxResolvedIndex>();

    mockIpcCommands({ [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(() => response.promise) });

    const service = await mockOpenedService();
    const opening = service.open("system.ltx");

    service.clear();
    response.resolve(getIndexOf([getEntryOf("a", "one.ltx")]));
    await opening;

    expect(service.entry).toBeNull();
    expect(service.index.isIdle).toBe(true);
    expect(service.index.value).toBeNull();
  });

  it("drops a page that lands after clear", async () => {
    const response = deferred<Array<LtxResolvedSection>>();

    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(getIndexOf([getEntryOf("a", "one.ltx")])),
      [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead(() => response.promise),
    });

    const service = await mockOpenedService();

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
    const response = deferred<Array<LtxResolvedSection>>();

    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(getIndexOf([getEntryOf("a", "one.ltx")])),
      [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead(() => response.promise),
    });

    const service = await mockOpenedService();

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
    const older = deferred<Array<LtxResolvedSection>>();
    const newer = deferred<Array<LtxResolvedSection>>();

    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(getIndexOf([getEntryOf("a", "one.ltx")])),
      [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead(() => older.promise),
    });

    const service = await mockOpenedService();

    await service.open("system.ltx");

    const first = service.request(["a"]);

    service.clear();
    await service.open("system.ltx");

    mockIpcCommands({ [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead(() => newer.promise) });

    const second = service.request(["a"]);

    older.resolve([]);
    await first;

    const repeated = service.request(["a"]);

    newer.resolve([getSectionOf("a")]);
    await Promise.all([second, repeated]);

    expect(mockInvoke.mock.calls.filter(([name]) => name === EConfigsCommand.READ_RESOLVED_SECTIONS)).toHaveLength(2);
    expect(service.sections.get("a")).toEqual(getSectionOf("a"));
  });

  it("accepts independent pages of the current resolution in either completion order", async () => {
    const first = deferred<Array<LtxResolvedSection>>();
    const second = deferred<Array<LtxResolvedSection>>();

    mockIpcCommands({
      [EConfigsCommand.LIST_RESOLVED_SECTIONS]: mockSessionRead(
        getIndexOf([getEntryOf("a", "one.ltx"), getEntryOf("b", "one.ltx")])
      ),
      [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead(() => first.promise),
    });

    const service = await mockOpenedService();

    await service.open("system.ltx");

    const firstPage = service.request(["a"]);

    mockIpcCommands({ [EConfigsCommand.READ_RESOLVED_SECTIONS]: mockSessionRead(() => second.promise) });

    const secondPage = service.request(["b"]);

    second.resolve([getSectionOf("b")]);
    await secondPage;
    first.resolve([getSectionOf("a")]);
    await firstPage;

    expect(service.sections.get("a")).toEqual(getSectionOf("a"));
    expect(service.sections.get("b")).toEqual(getSectionOf("b"));
  });
});
