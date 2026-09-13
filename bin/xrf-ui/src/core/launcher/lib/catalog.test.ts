import { describe, expect, it } from "@jest/globals";

import {
  getCatalogSummary,
  ICatalogEntry,
  ICatalogSection,
  toCatalogGroupFilters,
  toCatalogSearchText,
  toCatalogSecondaryText,
  toCatalogSections,
  toRankedSections,
} from "@/core/launcher/lib/catalog";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
  IApplicationGroup,
} from "@/core/routing/application";

function mockGroup(id: EApplicationGroupId, label: string): IApplicationGroup {
  return { accent: { light: "#000000", dark: "#ffffff" }, id, icon: null as never, label };
}

function mockApplication(
  id: EApplicationId,
  group: EApplicationGroupId,
  status: EApplicationStatus = EApplicationStatus.READY
): IApplicationDescriptor {
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

const ARCHIVES: IApplicationGroup = mockGroup(EApplicationGroupId.ARCHIVES, "Archives");
const SPAWNS: IApplicationGroup = mockGroup(EApplicationGroupId.SPAWNS, "Spawns");
const TEXTURES: IApplicationGroup = mockGroup(EApplicationGroupId.TEXTURES, "Textures");

const APPLICATIONS: Array<IApplicationDescriptor> = [
  mockApplication(EApplicationId.SPAWN_EDITOR, EApplicationGroupId.SPAWNS),
  mockApplication(EApplicationId.ARCHIVES_EXPLORER, EApplicationGroupId.ARCHIVES),
  mockApplication(EApplicationId.ARCHIVES_PACKER, EApplicationGroupId.ARCHIVES, EApplicationStatus.PLANNED),
];

describe("toCatalogSections", () => {
  it("files tools under their group, in the order the groups were given", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES, SPAWNS]);

    expect(sections.map(({ group }: ICatalogSection) => group?.id)).toEqual([
      EApplicationGroupId.ARCHIVES,
      EApplicationGroupId.SPAWNS,
    ]);
    expect(sections[0].entries.map(({ application }: ICatalogEntry) => application.id)).toEqual([
      EApplicationId.ARCHIVES_EXPLORER,
      EApplicationId.ARCHIVES_PACKER,
    ]);
  });

  it("drops a group with nothing in it, which would otherwise head an empty run", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES, TEXTURES, SPAWNS]);

    expect(sections.map(({ group }: ICatalogSection) => group?.id)).toEqual([
      EApplicationGroupId.ARCHIVES,
      EApplicationGroupId.SPAWNS,
    ]);
  });

  it("carries the group on every entry, which a ranked result no longer implies", () => {
    const [section]: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES]);

    expect(section.entries.every(({ group }: ICatalogEntry) => group === ARCHIVES)).toBe(true);
  });
});

describe("toRankedSections", () => {
  it("gathers results into one section that carries no heading", () => {
    const [section]: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES]);

    expect(toRankedSections(section.entries)).toEqual([{ group: null, entries: section.entries }]);
  });

  it("returns no section at all when nothing matched, rather than an empty one to draw", () => {
    expect(toRankedSections([])).toEqual([]);
  });
});

describe("getCatalogSummary", () => {
  it("counts the tools, the ready ones, and the groups they came from", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES, SPAWNS]);

    expect(getCatalogSummary(sections, null)).toBe("3 tools · 2 ready · 2 groups");
  });

  it("drops the group count once one group is chosen, which its own chip already says", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES]);

    expect(getCatalogSummary(sections, EApplicationGroupId.ARCHIVES)).toBe("2 tools · 1 ready");
  });

  it("says tool and group in the singular", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(
      [mockApplication(EApplicationId.SPAWN_EDITOR, EApplicationGroupId.SPAWNS)],
      [SPAWNS]
    );

    expect(getCatalogSummary(sections, null)).toBe("1 tool · 1 ready · 1 group");
  });
});

describe("catalog search text", () => {
  it("ranks an entry by its label alone", () => {
    const [{ entries }]: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [SPAWNS]);

    expect(toCatalogSearchText(entries[0])).toBe(EApplicationId.SPAWN_EDITOR);
  });

  it("matches the description and the group name without ranking by them", () => {
    const [{ entries }]: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [SPAWNS]);

    expect(toCatalogSecondaryText(entries[0])).toBe(`${EApplicationId.SPAWN_EDITOR} description Spawns`);
  });
});

describe("toCatalogGroupFilters", () => {
  it("counts each headed section for its chip", () => {
    const sections: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES, SPAWNS]);

    expect(toCatalogGroupFilters(sections)).toEqual([
      { group: ARCHIVES, count: 2 },
      { group: SPAWNS, count: 1 },
    ]);
  });

  it("offers no chip for a headless section, which names no group to choose", () => {
    const [section]: Array<ICatalogSection> = toCatalogSections(APPLICATIONS, [ARCHIVES]);

    expect(toCatalogGroupFilters(toRankedSections(section.entries))).toEqual([]);
  });
});
