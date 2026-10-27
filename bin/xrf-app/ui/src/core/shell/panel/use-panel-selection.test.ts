import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { IEditorPanel } from "@/core/shell/editor-shell";
import { IPanelSelection, usePanelSelection } from "@/core/shell/panel/use-panel-selection";
import { clearLocalStorage, setLocalStorageValue } from "@/core/storage";

function toPanel(id: string): IEditorPanel {
  return { icon: null, id, label: id, render: () => null };
}

function toRightPanels(editorPanelId: string): Array<IEditorPanel> {
  return [toPanel(editorPanelId), toPanel("notifications")];
}

function renderRightSelection(editorPanelId: string = "bones") {
  return renderHook(
    ({ panels }: { panels: Array<IEditorPanel> }): IPanelSelection => usePanelSelection("right", panels, "global"),
    { initialProps: { panels: toRightPanels(editorPanelId) } }
  );
}

describe("usePanelSelection", () => {
  beforeEach(() => {
    // The selection is remembered, so a leftover choice would decide the next case.
    clearLocalStorage();
  });

  it("opens the first default-open panel until one is chosen", () => {
    expect(renderRightSelection().result.current.activePanelId).toBe("bones");
  });

  it("selects the notification centre over an editor's default-open panel", () => {
    const { result } = renderRightSelection();

    act(() => result.current.onTogglePanel("notifications"));

    expect(result.current.activePanelId).toBe("notifications");
  });

  it("selects an editor panel while the notification centre is open", () => {
    const { result } = renderRightSelection();

    act(() => result.current.onTogglePanel("notifications"));
    act(() => result.current.onTogglePanel("bones"));

    expect(result.current.activePanelId).toBe("bones");
  });

  it("collapses the side when its open panel is toggled again, and keeps it closed", () => {
    const { result, rerender } = renderRightSelection();

    act(() => result.current.onTogglePanel("notifications"));
    act(() => result.current.onTogglePanel("notifications"));

    expect(result.current.activePanel).toBeNull();

    // A side closed by hand stays closed rather than falling back to the editor's default-open panel.
    rerender({ panels: toRightPanels("bones") });

    expect(result.current.activePanel).toBeNull();
  });

  it("restores the remembered selection over an editor's default-open panel", () => {
    setLocalStorageValue("xrf.panels.right.global", "notifications");

    expect(renderRightSelection().result.current.activePanelId).toBe("notifications");
  });

  it("keeps the notification centre open when another tool publishes its own panels", () => {
    const { result, rerender } = renderRightSelection("bones");

    act(() => result.current.onTogglePanel("notifications"));
    rerender({ panels: toRightPanels("header") });

    expect(result.current.activePanelId).toBe("notifications");
  });
});
