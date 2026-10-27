import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { EToolId } from "@/core/routing/tool";
import { TShellContributionTool } from "@/core/shell/contributions/shell-contributions";
import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { IPanelSelection, usePanelSelection } from "@/core/shell/panels/use-panel-selection";
import { clearLocalStorage, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";

function toPanel(id: string, isOpenByDefault?: boolean): IShellPanel {
  return { icon: null, id, isOpenByDefault, label: id, render: () => null };
}

function toRightPanels(toolPanelId: string): Array<IShellPanel> {
  return [toPanel(toolPanelId), toPanel("notifications")];
}

interface ISelectionProps {
  panels: Array<IShellPanel>;
  tool: TShellContributionTool;
}

function renderRightSelection(toolPanelId: string = "bones", tool: TShellContributionTool = EToolId.VISUALS_EXPLORER) {
  return renderHook(({ panels, tool }: ISelectionProps): IPanelSelection => usePanelSelection("right", panels, tool), {
    initialProps: { panels: toRightPanels(toolPanelId), tool },
  });
}

describe("usePanelSelection", () => {
  beforeEach(() => {
    // The selection is remembered, so a leftover choice would decide the next case.
    clearLocalStorage();
  });

  it("opens the first default-open panel until one is chosen", () => {
    expect(renderRightSelection().result.current.activePanelId).toBe("bones");
  });

  it("selects the notification centre over a tool's default-open panel", () => {
    const { result } = renderRightSelection();

    act(() => result.current.onTogglePanel("notifications"));

    expect(result.current.activePanelId).toBe("notifications");
  });

  it("selects a tool's panel while the notification centre is open", () => {
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

    // A side closed by hand stays closed rather than falling back to the tool's default-open panel.
    rerender({ panels: toRightPanels("bones"), tool: EToolId.VISUALS_EXPLORER });

    expect(result.current.activePanel).toBeNull();
  });

  it("restores the tool's remembered selection over its default-open panel", () => {
    setLocalStorageValue("xrf.panels.right.visuals-explorer", "notifications");

    expect(renderRightSelection().result.current.activePanelId).toBe("notifications");
  });

  it("remembers each tool's choice apart, so another tool still opens its own default", () => {
    const { result, rerender } = renderRightSelection("bones", EToolId.VISUALS_EXPLORER);

    act(() => result.current.onTogglePanel("notifications"));
    rerender({ panels: toRightPanels("findings"), tool: EToolId.DIALOGS_EDITOR });

    expect(result.current.activePanelId).toBe("findings");

    rerender({ panels: toRightPanels("bones"), tool: EToolId.VISUALS_EXPLORER });

    expect(result.current.activePanelId).toBe("notifications");
    expect(getLocalStorageValue("xrf.panels.right.visuals-explorer")).toBe("notifications");
  });

  it("reads nothing from the choice once kept for every tool at once", () => {
    setLocalStorageValue("xrf.panels.right.global", "notifications");

    expect(renderRightSelection().result.current.activePanelId).toBe("bones");
  });
});
