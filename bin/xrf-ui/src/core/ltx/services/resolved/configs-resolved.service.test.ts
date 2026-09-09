import { beforeEach, describe, expect, it } from "@jest/globals";

import { ConfigsProjectDescriptor } from "@/core/bindings/types/xrf-app";
import { LtxResolvedIndex, LtxResolvedSection } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

/** One index entry, with only the parts a case is about spelled out. */
function entryOf(name: string, origin: string): LtxResolvedIndex["sections"][number] {
  return { fieldCount: 1, name, origin, parents: [] };
}

function indexOf(sections: LtxResolvedIndex["sections"]): LtxResolvedIndex {
  return { dialect: "ltx", diagnostics: [], entry: "system.ltx", sections };
}

function sectionOf(name: string): LtxResolvedSection {
  return { entry: "system.ltx", fields: [], name, origin: "items\\w_base.ltx", parents: [] };
}

/** A service whose project is already open, since every read is addressed by that session. */
function openedService(): ConfigsResolvedService {
  const { service, container } = mockInjectedService(ConfigsResolvedService, [ConfigsProjectService]);
  const project = container.get(ConfigsProjectService);

  project.project = project.project.asReady({ sessionId: "session-1" } as ConfigsProjectDescriptor);

  return service;
}

describe("ConfigsResolvedService", () => {
  beforeEach(() => setMockInvokeResponses({}));

  it("indexes an entry point once and answers the second ask from what it holds", async () => {
    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": indexOf([entryOf("wpn_base", "items\\w_base.ltx")]),
    });

    const service: ConfigsResolvedService = openedService();

    await service.open("system.ltx");
    await service.open("system.ltx");

    expect(mockInvoke.mock.calls.filter(([name]) => name === "plugin:configs|list_resolved_sections")).toHaveLength(1);
    expect(service.index.value?.sections).toHaveLength(1);
  });

  it("asks only for the sections it does not already hold", async () => {
    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": indexOf([entryOf("a", "one.ltx"), entryOf("b", "one.ltx")]),
      "plugin:configs|read_resolved_sections": [sectionOf("a")],
    });

    const service: ConfigsResolvedService = openedService();

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
      "plugin:configs|list_resolved_sections": indexOf([entryOf("a", "one.ltx")]),
      "plugin:configs|read_resolved_sections": [sectionOf("a")],
    });

    const service: ConfigsResolvedService = openedService();

    await service.open("system.ltx");

    const page: Promise<void> = service.request(["a"]);

    // Reopening while the page is in flight: the answer describes a document nobody is looking at any more.
    await service.open("other.ltx");
    await page;

    expect(service.sections.has("a")).toBe(false);
  });

  it("narrows to what one config declared, and widens back", async () => {
    setMockInvokeResponses({
      "plugin:configs|list_resolved_sections": indexOf([
        entryOf("wpn_base", "items\\w_base.ltx"),
        entryOf("system", "system.ltx"),
      ]),
    });

    const service: ConfigsResolvedService = openedService();

    await service.open("system.ltx");

    expect(service.visibleSections).toHaveLength(2);

    service.narrowTo("items\\w_base.ltx");

    // What a person opening an included config wants: what that file's sections came to, not where they sit among the
    // thousands its entry point resolves.
    expect(service.visibleSections.map((section) => section.name)).toEqual(["wpn_base"]);

    service.narrowTo(null);

    expect(service.visibleSections).toHaveLength(2);
  });
});
