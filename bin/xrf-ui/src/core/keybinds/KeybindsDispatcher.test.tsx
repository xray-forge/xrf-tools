import { describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent } from "@testing-library/react";
import { Container, Injectable } from "@wirestate/core";

import { RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND } from "@/applications/sprite-equipment-editor/commands";
import { KeybindCommand } from "@/core/commands";
import { HelpService } from "@/core/help/services/help";
import { KeybindsDispatcher } from "@/core/keybinds/KeybindsDispatcher";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";
import { SettingsService } from "@/core/settings/services/settings";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Logger } from "@/lib/logging";

/** Stands in for the service a screen binds to answer the search chord. */
@Injectable()
class SearchFixtureService {
  public calls: number = 0;

  @KeybindCommand(FOCUS_SEARCH_KEYBIND_COMMAND)
  public focus(): void {
    this.calls += 1;
  }
}

@Injectable()
class ReloadFixtureService {
  public calls: number = 0;

  @KeybindCommand(RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND)
  public reload(): void {
    this.calls += 1;
  }
}

interface IDispatcherHarness {
  container: Container;
  field: HTMLElement;
  guarded: HTMLElement;
  search: SearchFixtureService;
}

function renderDispatcher(route: string = "/archives-explorer"): IDispatcherHarness {
  const container: Container = mockContainer([SearchFixtureService]);

  const { getByLabelText } = renderWithProviders(
    <>
      <input aria-label={"field"} />
      <div aria-label={"guarded"} onKeyDown={(event) => event.stopPropagation()} />
      <KeybindsDispatcher />
    </>,
    { container, route }
  );

  return {
    container,
    field: getByLabelText("field"),
    guarded: getByLabelText("guarded"),
    search: container.get(SearchFixtureService),
  };
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
    const { field, search } = renderDispatcher();

    fireEvent.keyDown(field, { key: "/" });

    expect(search.calls).toBe(0);

    fireEvent.keyDown(field, { ctrlKey: true, key: "k" });

    expect(search.calls).toBe(1);
  });

  it("runs a bare chord outside a text field", () => {
    const { search } = renderDispatcher();

    fireEvent.keyDown(document.body, { key: "/" });

    expect(search.calls).toBe(1);
  });

  it("yields to a control that handles the key itself", () => {
    const { guarded, search } = renderDispatcher();

    fireEvent.keyDown(guarded, { ctrlKey: true, key: "k" });

    expect(search.calls).toBe(0);
  });

  it("ignores a repeat, so a held key is not an action run sixty times", () => {
    const { search } = renderDispatcher();

    fireEvent.keyDown(document.body, { ctrlKey: true, key: "k", repeat: true });

    expect(search.calls).toBe(0);
  });

  it("stays out of the way while a modal has the screen", () => {
    const { search } = renderDispatcher();

    const dialog: HTMLElement = document.createElement("div");

    dialog.setAttribute("aria-modal", "true");
    document.body.appendChild(dialog);

    const event: KeyboardEvent = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "/" });

    fireEvent(document.body, event);

    expect(search.calls).toBe(0);
    expect(event.defaultPrevented).toBe(false);

    dialog.remove();
  });

  it("warns in dev mode about a chord nothing implements, and not about one a guard refused", () => {
    const warn = jest.spyOn(Logger, "warn").mockImplementation(() => undefined);

    try {
      // No fixture service, so the archives explorer's search chord resolves to a command with no handler at all.
      const container: Container = mockContainer();

      renderWithProviders(<KeybindsDispatcher />, { container, route: "/archives-explorer" });

      act(() => container.get(SettingsService).setDevModeEnabled(true));
      act(() => container.get(HelpService).setApplication("archives-explorer", false));

      fireEvent.keyDown(document.body, { key: "/" });

      expect(warn).toHaveBeenCalledTimes(1);

      // Help is implemented and its guard refuses, which is enablement working rather than anything to report.
      fireEvent.keyDown(document.body, { key: "F1" });

      expect(warn).toHaveBeenCalledTimes(1);
    } finally {
      warn.mockRestore();
    }
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
