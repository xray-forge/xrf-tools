import { describe, expect, it } from "@jest/globals";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor, IToolGroup } from "@/core/routing/tool";

import {
  getCatalogSummary,
  ICatalogEntry,
  ICatalogSection,
  toCatalogGroupFilters,
  toCatalogSearchText,
  toCatalogSecondaryText,
  toCatalogSections,
  toRankedSections,
} from "./catalog";

function mockGroup(id: EToolGroupId, label: string): IToolGroup {
  return { accent: { light: "#000000", dark: "#ffffff" }, id, icon: null as never, label };
}

function mockTool(id: EToolId, group: EToolGroupId, status: EToolStatus = EToolStatus.READY): IToolDescriptor {
  return {
    Component: () => null,
    description: `${id} description`,
    group,
    icon: null as never,
    id,
    label: id,
    path: `/${id}`,
    status,
  };
}

const ARCHIVES: IToolGroup = mockGroup(EToolGroupId.ARCHIVES, "Archives");
const SPAWNS: IToolGroup = mockGroup(EToolGroupId.SPAWNS, "Spawns");
const TEXTURES: IToolGroup = mockGroup(EToolGroupId.TEXTURES, "Textures");

const TOOLS: Array<IToolDescriptor> = [
  mockTool(EToolId.SPAWN_EDITOR, EToolGroupId.SPAWNS),
  mockTool(EToolId.ARCHIVES_EXPLORER, EToolGroupId.ARCHIVES),
  mockTool(EToolId.ARCHIVES_PACKER, EToolGroupId.ARCHIVES, EToolStatus.PLANNED),
];

describe("toCatalogSections", () => {
  it("files tools under their group, in the order the groups were given", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES, SPAWNS]);

    expect(sections.map(({ group }: ICatalogSection) => group?.id)).toEqual([
      EToolGroupId.ARCHIVES,
      EToolGroupId.SPAWNS,
    ]);
    expect(sections[0].entries.map(({ tool }: ICatalogEntry) => tool.id)).toEqual([
      EToolId.ARCHIVES_EXPLORER,
      EToolId.ARCHIVES_PACKER,
    ]);
  });

  it("drops a group with nothing in it, which would otherwise head an empty run", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES, TEXTURES, SPAWNS]);

    expect(sections.map(({ group }: ICatalogSection) => group?.id)).toEqual([
      EToolGroupId.ARCHIVES,
      EToolGroupId.SPAWNS,
    ]);
  });

  it("carries the group on every entry, which a ranked result no longer implies", () => {
    const [section]: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES]);

    expect(section.entries.every(({ group }: ICatalogEntry) => group === ARCHIVES)).toBe(true);
  });
});

describe("toRankedSections", () => {
  it("gathers results into one section that carries no heading", () => {
    const [section]: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES]);

    expect(toRankedSections(section.entries)).toEqual([{ group: null, entries: section.entries }]);
  });

  it("returns no section at all when nothing matched, rather than an empty one to draw", () => {
    expect(toRankedSections([])).toEqual([]);
  });
});

describe("getCatalogSummary", () => {
  it("counts the tools and the groups they came from", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES, SPAWNS]);

    expect(getCatalogSummary(sections, null)).toBe("3 tools · 2 groups");
  });

  it("drops the group count once one group is chosen, which its own chip already says", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES]);

    expect(getCatalogSummary(sections, EToolGroupId.ARCHIVES)).toBe("2 tools");
  });

  it("says tool and group in the singular", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(
      [mockTool(EToolId.SPAWN_EDITOR, EToolGroupId.SPAWNS)],
      [SPAWNS]
    );

    expect(getCatalogSummary(sections, null)).toBe("1 tool · 1 group");
  });
});

describe("catalog search text", () => {
  it("ranks an entry by its label alone", () => {
    const [{ entries }]: Array<ICatalogSection> = toCatalogSections(TOOLS, [SPAWNS]);

    expect(toCatalogSearchText(entries[0])).toBe(EToolId.SPAWN_EDITOR);
  });

  it("matches the description and the group name without ranking by them", () => {
    const [{ entries }]: Array<ICatalogSection> = toCatalogSections(TOOLS, [SPAWNS]);

    expect(toCatalogSecondaryText(entries[0])).toBe(`${EToolId.SPAWN_EDITOR} description Spawns`);
  });
});

describe("toCatalogGroupFilters", () => {
  it("counts each headed section for its chip", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES, SPAWNS]);

    expect(toCatalogGroupFilters(sections)).toEqual([
      { group: ARCHIVES, count: 2 },
      { group: SPAWNS, count: 1 },
    ]);
  });

  it("offers no chip for a headless section, which names no group to choose", () => {
    const [section]: Array<ICatalogSection> = toCatalogSections(TOOLS, [ARCHIVES]);

    expect(toCatalogGroupFilters(toRankedSections(section.entries))).toEqual([]);
  });
});
