import { describe, expect, it, jest } from "@jest/globals";

import { IEditorPanel } from "@/core/shell/editor-shell/editor-panel";
import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";
import { mockInjectedService } from "@/fixtures/utils/container";

import { EditorShellService } from "./editor-shell.service";

const FIRST: string = "/translations-editor";
const SECOND: string = "/dialogs-editor";

function panel(id: string): IEditorPanel {
  return { id, label: id, icon: null, render: jest.fn(() => null) };
}

describe("EditorShellService", () => {
  it("hides outgoing contributions before the next application publishes", () => {
    const { service } = mockInjectedService(EditorShellService);

    service.publishStatus("old-status", FIRST, ["3 files"]);
    service.publishPanels("old-panels", FIRST, [panel("files")]);

    expect(service.getStatus(SECOND)).toEqual([]);
    expect(service.getPanels(SECOND)).toEqual([]);
    expect(service.getStatus(FIRST)).toEqual(["3 files"]);
    expect(service.getPanels(FIRST)).toHaveLength(1);
  });

  it.each([FIRST, SECOND])("does not let old cleanup erase new publications for %s", (application) => {
    const { service } = mockInjectedService(EditorShellService);
    const current = panel("current");

    service.publishStatus("old-status", FIRST, ["old"]);
    service.publishPanels("old-panels", FIRST, [panel("old")]);
    service.publishStatus("new-status", application, ["current"]);
    service.publishPanels("new-panels", application, [current]);
    service.releaseStatus("old-status");
    service.releasePanels("old-panels");

    expect(service.getStatus(application)).toEqual(["current"]);
    expect(service.getPanels(application)).toEqual([current]);

    service.releaseStatus("new-status");
    service.releasePanels("new-panels");

    expect(service.getStatus(application)).toEqual([]);
    expect(service.getPanels(application)).toEqual([]);
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

  // The status bar's memory hover is the shell's, and each application adds its own rows to it while it is shown.
  it("holds the memory hover rows of the application shown alone, until their owner lets them go", () => {
    const { service } = mockInjectedService(EditorShellService);

    function row(): IEditorStatusDetail {
      return { label: "Renderer copies", value: "1 MB" };
    }

    service.publishMemoryDetails("level", FIRST, [row]);

    expect(service.getMemoryDetails(FIRST)).toEqual([row]);
    expect(service.getMemoryDetails(SECOND)).toEqual([]);

    service.releaseMemoryDetails("other");

    expect(service.getMemoryDetails(FIRST)).toEqual([row]);

    service.releaseMemoryDetails("level");

    expect(service.getMemoryDetails(FIRST)).toEqual([]);
  });
});
