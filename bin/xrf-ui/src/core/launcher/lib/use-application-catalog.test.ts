import { describe, expect, it, jest } from "@jest/globals";
import { act, renderHook, RenderHookResult } from "@testing-library/react";

import { ICatalogEntry, ICatalogSection } from "@/core/launcher/lib/catalog";
import { IUseApplicationCatalog, useApplicationCatalog } from "@/core/launcher/lib/use-application-catalog";
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

function mockApplication(id: EApplicationId, group: EApplicationGroupId, label: string): IApplicationDescriptor {
  return {
    Component: () => null,
    description: `${label} description`,
    group,
    icon: null as never,
    id,
    label,
    path: `/${id}`,
    status: EApplicationStatus.READY,
  };
}

const GROUPS: Array<IApplicationGroup> = [
  mockGroup(EApplicationGroupId.ARCHIVES, "Archives"),
  mockGroup(EApplicationGroupId.SPAWNS, "Spawns"),
];

const APPLICATIONS: Array<IApplicationDescriptor> = [
  mockApplication(EApplicationId.ARCHIVES_EXPLORER, EApplicationGroupId.ARCHIVES, "Archives explorer"),
  mockApplication(EApplicationId.ARCHIVES_PACKER, EApplicationGroupId.ARCHIVES, "Archives packer"),
  mockApplication(EApplicationId.SPAWN_EDITOR, EApplicationGroupId.SPAWNS, "Spawn editor"),
];

function renderCatalog(
  onSelect: (entry: ICatalogEntry) => void = jest.fn()
): RenderHookResult<IUseApplicationCatalog, unknown> {
  return renderHook(() => useApplicationCatalog({ applications: APPLICATIONS, groups: GROUPS, onSelect }));
}

/** Tool labels in the order the sections hold them, which is the order the body draws. */
function getLabels(sections: ReadonlyArray<ICatalogSection>): Array<string> {
  return sections.flatMap((section: ICatalogSection) =>
    section.entries.map(({ application }: ICatalogEntry) => application.label)
  );
}

describe("useApplicationCatalog", () => {
  it("opens on every group, each section headed by the group it holds", () => {
    const { result } = renderCatalog();

    expect(result.current.sections.map((section: ICatalogSection) => section.group?.label)).toEqual([
      "Archives",
      "Spawns",
    ]);
    expect(getLabels(result.current.sections)).toEqual(["Archives explorer", "Archives packer", "Spawn editor"]);
  });

  it("narrows the sections to the chosen group, and lets go again", () => {
    const { result } = renderCatalog();

    act(() => result.current.onSelectGroup(EApplicationGroupId.SPAWNS));

    expect(getLabels(result.current.sections)).toEqual(["Spawn editor"]);
    expect(result.current.summary).toBe("1 tool · 1 ready");

    act(() => result.current.onSelectGroup(null));

    expect(getLabels(result.current.sections)).toEqual(["Archives explorer", "Archives packer", "Spawn editor"]);
  });

  it("keeps every chip on offer while narrowed, so the counts still show what was set aside", () => {
    const { result } = renderCatalog();

    act(() => result.current.onSelectGroup(EApplicationGroupId.SPAWNS));

    expect(result.current.filters.map(({ group }) => group.label)).toEqual(["Archives", "Spawns"]);
  });

  it("ranks a query into the one section that carries no heading", () => {
    const { result } = renderCatalog();

    act(() => result.current.search.setQuery("archives"));

    expect(result.current.sections).toHaveLength(1);
    expect(result.current.sections[0].group).toBeNull();
    expect(getLabels(result.current.sections)).toEqual(["Archives explorer", "Archives packer"]);
  });

  it("searches inside the chosen group rather than past it", () => {
    const { result } = renderCatalog();

    act(() => result.current.onSelectGroup(EApplicationGroupId.SPAWNS));
    act(() => result.current.search.setQuery("archives"));

    // "Archives explorer" matches the query and is deliberately out of reach: the chip narrowed first.
    expect(result.current.sections).toEqual([]);
  });

  it("leaves no section to draw when nothing matches, rather than an empty one", () => {
    const { result } = renderCatalog();

    act(() => result.current.search.setQuery("qqq"));

    expect(result.current.sections).toEqual([]);
    expect(result.current.search.total).toBe(0);
  });

  it("counts the catalog it is showing, not the one it came from", () => {
    const { result } = renderCatalog();

    expect(result.current.summary).toBe("3 tools · 3 ready · 2 groups");
  });

  it("hands an accepted result back whole, group included", () => {
    const onSelect = jest.fn();
    const { result } = renderCatalog(onSelect);

    act(() => result.current.search.setQuery("spawn"));
    act(() => result.current.search.onInputKeyDown({ key: "Enter", preventDefault: jest.fn() } as never));

    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect).toHaveBeenCalledWith({
      application: APPLICATIONS[2],
      group: GROUPS[1],
    });
  });
});
