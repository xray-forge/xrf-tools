import { describe, expect, it, jest } from "@jest/globals";

import { EToolId } from "@/core/routing/tool";
import { defineMenu, IMenu, IMenuContribution } from "@/core/shell/menu/lib";
import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ShellContributionsService } from "./shell-contributions.service";

const TOOL: EToolId = EToolId.TRANSLATIONS_EDITOR;

const MENU: IMenu<"only"> = defineMenu({ id: "test/menu", label: "Menu", sections: ["only"] });

const PANEL: IShellPanel = { id: "files", label: "Files", icon: null, render: jest.fn(() => null) };

const MENU_CONTRIBUTION: IMenuContribution = {
  section: MENU.sections.only,
  entries: [{ kind: "action", id: "test/act", label: "Act", onSelect: jest.fn() }],
};

describe("ShellContributionsService", () => {
  it("keeps a detailed status segment drawn until one of its rows changes", () => {
    const { service } = mockInjectedService(ShellContributionsService);

    service.status.publish("owner", TOOL, [
      "3 files",
      { id: "memory", text: "350 MB", details: [{ label: "Backend", value: "350 MB" }] },
    ]);

    const original = service.status.get(TOOL)[1];

    service.status.publish("owner", TOOL, [
      "3 files",
      { id: "memory", text: "350 MB", details: [{ label: "Backend", value: "350 MB" }] },
    ]);

    expect(service.status.get(TOOL)[1]).toBe(original);

    service.status.publish("owner", TOOL, [
      "3 files",
      { id: "memory", text: "350 MB", details: [{ label: "Backend", value: "351 MB" }] },
    ]);

    expect(service.status.get(TOOL)[1]).toEqual({
      id: "memory",
      text: "350 MB",
      details: [{ label: "Backend", value: "351 MB" }],
    });
  });

  it("clears every part when the root service is deprovisioned", () => {
    const { service } = mockInjectedService(ShellContributionsService);

    service.panels.publish("panels", TOOL, [PANEL]);
    service.status.publish("status", TOOL, ["3 files"]);
    service.menus.publish("menus", TOOL, [MENU_CONTRIBUTION]);
    service.onDeprovision();

    expect(service.panels.get(TOOL)).toEqual([]);
    expect(service.status.get(TOOL)).toEqual([]);
    expect(service.menus.get(TOOL)).toEqual([]);
  });
});
