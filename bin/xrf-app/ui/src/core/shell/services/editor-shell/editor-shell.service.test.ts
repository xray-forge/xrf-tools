import { describe, expect, it, jest } from "@jest/globals";

import { IEditorPanel } from "@/core/shell/editor-shell/editor-panel";
import { defineMenu, IMenu, IMenuContribution } from "@/core/shell/menu/lib";
import { mockInjectedService } from "@/fixtures/utils/container";

import { EditorShellService } from "./editor-shell.service";

const FIRST: string = "/translations-editor";
const SECOND: string = "/dialogs-editor";

const MENU: IMenu<"only"> = defineMenu({ id: "test/menu", label: "Menu", sections: ["only"] });

function contribution(id: string): IMenuContribution {
  return { section: MENU.sections.only, entries: [{ kind: "action", id, label: id, onSelect: jest.fn() }] };
}

function panel(id: string): IEditorPanel {
  return { id, label: id, icon: null, render: jest.fn(() => null) };
}

describe("EditorShellService", () => {
  it("hides outgoing contributions before the next tool publishes", () => {
    const { service } = mockInjectedService(EditorShellService);

    service.publishStatus("old-status", FIRST, ["3 files"]);
    service.publishPanels("old-panels", FIRST, [panel("files")]);

    expect(service.getStatus(SECOND)).toEqual([]);
    expect(service.getPanels(SECOND)).toEqual([]);
    expect(service.getStatus(FIRST)).toEqual(["3 files"]);
    expect(service.getPanels(FIRST)).toHaveLength(1);
  });

  it.each([FIRST, SECOND])("does not let old cleanup erase new publications for %s", (tool) => {
    const { service } = mockInjectedService(EditorShellService);
    const current = panel("current");

    service.publishStatus("old-status", FIRST, ["old"]);
    service.publishPanels("old-panels", FIRST, [panel("old")]);
    service.publishStatus("new-status", tool, ["current"]);
    service.publishPanels("new-panels", tool, [current]);
    service.releaseStatus("old-status");
    service.releasePanels("old-panels");

    expect(service.getStatus(tool)).toEqual(["current"]);
    expect(service.getPanels(tool)).toEqual([current]);

    service.releaseStatus("new-status");
    service.releasePanels("new-panels");

    expect(service.getStatus(tool)).toEqual([]);
    expect(service.getPanels(tool)).toEqual([]);
  });

  it("keeps equal status text stable while still transferring ownership", () => {
    const { service } = mockInjectedService(EditorShellService);

    service.publishStatus("old", FIRST, ["3 files", "12 objects"]);

    const original = service.getStatus(FIRST);

    service.publishStatus("old", FIRST, ["3 files", "12 objects"]);

    expect(service.getStatus(FIRST)).toBe(original);

    service.publishStatus("new", FIRST, ["3 files", "12 objects"]);
    service.releaseStatus("old");

    expect(service.getStatus(FIRST)).toEqual(original);
  });

  it("keeps equal detailed segments stable and replaces them when a detail changes", () => {
    const { service } = mockInjectedService(EditorShellService);

    service.publishStatus("owner", FIRST, [
      "3 files",
      { id: "memory", text: "350 MB", details: [{ label: "Backend", value: "350 MB" }] },
    ]);

    const original = service.getStatus(FIRST);

    service.publishStatus("owner", FIRST, [
      "3 files",
      { id: "memory", text: "350 MB", details: [{ label: "Backend", value: "350 MB" }] },
    ]);

    expect(service.getStatus(FIRST)).toBe(original);

    service.publishStatus("owner", FIRST, [
      "3 files",
      { id: "memory", text: "350 MB", details: [{ label: "Backend", value: "351 MB" }] },
    ]);

    expect(service.getStatus(FIRST)).not.toBe(original);
    expect(service.getStatus(FIRST)[1]).toEqual({
      id: "memory",
      text: "350 MB",
      details: [{ label: "Backend", value: "351 MB" }],
    });
  });

  it("snapshots caller arrays without transforming panel descriptors or renderers", () => {
    const { service } = mockInjectedService(EditorShellService);
    const current = panel("current");
    const panels: Array<IEditorPanel> = [current];
    const segments: Array<string> = ["3 files"];

    service.publishPanels("panels", FIRST, panels);
    service.publishStatus("status", FIRST, segments);
    panels.length = 0;
    segments[0] = "mutated";

    expect(service.getStatus(FIRST)).toEqual(["3 files"]);
    expect(service.getPanels(FIRST)[0]).toBe(current);
    expect(service.getPanels(FIRST)[0].render).toBe(current.render);
    expect(current.render).not.toHaveBeenCalled();
  });

  it("clears publications when the root service is deprovisioned", () => {
    const { service } = mockInjectedService(EditorShellService);

    service.publishPanels("panels", FIRST, [panel("files")]);
    service.publishStatus("status", FIRST, ["3 files"]);
    service.onDeprovision();

    expect(service.getStatus(FIRST)).toEqual([]);
    expect(service.getPanels(FIRST)).toEqual([]);
  });

  it("holds menu entries from many owners at once, each keeping the place it first published at", () => {
    const { service } = mockInjectedService(EditorShellService);
    const shell = contribution("shell");
    const panelV1 = contribution("panel-v1");
    const panelV2 = contribution("panel-v2");
    const tool = contribution("tool");

    service.publishMenus("shell", FIRST, [shell]);
    service.publishMenus("panel", FIRST, [panelV1]);
    service.publishMenus("tool", FIRST, [tool]);
    service.publishMenus("panel", FIRST, [panelV2]);
    service.publishMenus("other", SECOND, [contribution("other")]);

    expect(service.getMenus(FIRST)).toEqual([shell, panelV2, tool]);

    service.releaseMenus("panel");

    expect(service.getMenus(FIRST)).toEqual([shell, tool]);
  });
});
