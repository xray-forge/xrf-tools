import { describe, expect, it, jest } from "@jest/globals";

import { IEditorPanel } from "@/core/shell/editor-shell/editor-panel";
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
});
