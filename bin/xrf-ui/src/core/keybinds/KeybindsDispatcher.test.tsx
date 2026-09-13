import { describe, expect, it } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { Container, Injectable } from "@wirestate/core";

import { RELOAD_EQUIPMENT_SPRITE_COMMAND } from "@/applications/sprite-equipment-editor/commands";
import { Command } from "@/core/commands";
import { HelpService } from "@/core/help/services/help";
import { KeybindsDispatcher } from "@/core/keybinds/KeybindsDispatcher";
import { LauncherSearchService } from "@/core/launcher/services/launcher-search";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

@Injectable()
class ReloadFixtureService {
  public calls: number = 0;

  @Command(RELOAD_EQUIPMENT_SPRITE_COMMAND)
  public reload(): void {
    this.calls += 1;
  }
}

interface IDispatcherHarness {
  container: Container;
  field: HTMLElement;
  guarded: HTMLElement;
}

function renderDispatcher(route: string = "/archives-explorer"): IDispatcherHarness {
  const container: Container = mockContainer();
  const { getByLabelText } = renderWithProviders(
    <>
      <input aria-label={"field"} />
      <div aria-label={"guarded"} onKeyDown={(event) => event.stopPropagation()} />
      <KeybindsDispatcher />
    </>,
    { container, route }
  );

  return { container, field: getByLabelText("field"), guarded: getByLabelText("guarded") };
}

describe("KeybindsDispatcher", () => {
  it("runs the command a chord names", () => {
    const { container } = renderDispatcher();
    const helpService: HelpService = container.get(HelpService);

    act(() => helpService.setApplication("archives-explorer", true));

    const event: KeyboardEvent = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "F1" });

    fireEvent(window, event);

    expect(helpService.isOpen).toBe(true);
    expect(event.defaultPrevented).toBe(true);
  });

  it("leaves the key alone when the command's own guard refuses it", () => {
    const { container } = renderDispatcher();
    const helpService: HelpService = container.get(HelpService);

    // The application authored no help, so nothing should consume F1 on its behalf.
    act(() => helpService.setApplication("characters-explorer", false));

    const event: KeyboardEvent = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "F1" });

    fireEvent(window, event);

    expect(helpService.isOpen).toBe(false);
    expect(event.defaultPrevented).toBe(false);
  });

  it("withholds a bare chord from a text field while a modified one still reaches it", () => {
    const { container, field } = renderDispatcher();
    const launcherSearchService: LauncherSearchService = container.get(LauncherSearchService);

    act(() => launcherSearchService.setMounted(true));

    fireEvent.keyDown(field, { key: "/" });

    expect(launcherSearchService.focusRevision).toBe(0);

    fireEvent.keyDown(field, { ctrlKey: true, key: "k" });

    expect(launcherSearchService.focusRevision).toBe(1);
  });

  it("runs a bare chord outside a text field", () => {
    const { container } = renderDispatcher();
    const launcherSearchService: LauncherSearchService = container.get(LauncherSearchService);

    act(() => launcherSearchService.setMounted(true));

    fireEvent.keyDown(document.body, { key: "/" });

    expect(launcherSearchService.focusRevision).toBe(1);
  });

  it("yields to a control that handles the key itself", () => {
    const { container, guarded } = renderDispatcher();
    const launcherSearchService: LauncherSearchService = container.get(LauncherSearchService);

    act(() => launcherSearchService.setMounted(true));

    fireEvent.keyDown(guarded, { ctrlKey: true, key: "k" });

    expect(launcherSearchService.focusRevision).toBe(0);
  });

  it("ignores a repeat, so a held key is not an action run sixty times", () => {
    const { container } = renderDispatcher();
    const launcherSearchService: LauncherSearchService = container.get(LauncherSearchService);

    act(() => launcherSearchService.setMounted(true));

    fireEvent.keyDown(document.body, { ctrlKey: true, key: "k", repeat: true });

    expect(launcherSearchService.focusRevision).toBe(0);
  });

  it.each([
    ["/sprite-equipment-editor", 1],
    ["/archives-explorer", 0],
  ])("reaches an application's own command at %s and nowhere else", (route: string, expected: number) => {
    // Left unprovisioned: the container provider provisions it, which is also what binds the fixture's handler.
    const container: Container = mockContainer([ReloadFixtureService]);

    renderWithProviders(<KeybindsDispatcher />, { container, route });
    fireEvent.keyDown(document.body, { key: "F5" });

    // The handler is bound either way; only the routed application's declarations put F5 in the keymap.
    expect(container.get(ReloadFixtureService).calls).toBe(expected);
  });
});
