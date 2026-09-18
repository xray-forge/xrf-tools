import { beforeEach, describe, expect, it } from "@jest/globals";

import { SPRITE_EQUIPMENT_EDITOR_APPLICATION } from "@/applications/sprite-equipment-editor";
import { RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND } from "@/applications/sprite-equipment-editor/commands";
import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";
import { ROOT_KEYBIND_COMMANDS } from "@/core/commands/root-commands";
import { IKeybinding, IKeymap } from "@/core/keybinds/lib/keymap";
import { LAUNCHER_KEYBIND_COMMANDS } from "@/core/launcher/commands";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";
import { KEYBINDS_STORAGE_KEY } from "@/core/storage";
import { mockInjectedService } from "@/fixtures/utils/container";

import { KeymapService } from "./keymap.service";

const RELOAD_ID: string = RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND.id;

function toIds(commands: ReadonlyArray<IKeybindCommand>): Array<string> {
  return commands.map((command: IKeybindCommand) => command.id);
}

const RELOAD: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.APPLICATION,
  chords: ["F5"],
  description: "Reload.",
  id: "fixture/reload",
  label: "Reload",
});

describe("KeymapService", () => {
  // The service reads storage once, when it is constructed, so each case needs its own starting state.
  beforeEach(() => window.localStorage.clear());

  it("answers with the declared chords when nothing was rebound", () => {
    const { service } = mockInjectedService(KeymapService);

    expect(service.getChords(RELOAD)).toEqual(["F5"]);
  });

  it("prefers a stored override over the declaration", () => {
    window.localStorage.setItem(KEYBINDS_STORAGE_KEY, JSON.stringify({ "fixture/reload": ["mod+r"] }));

    const { service } = mockInjectedService(KeymapService);

    expect(service.getChords(RELOAD)).toEqual(["mod+r"]);
  });

  it("discards an unusable entry without losing the entries beside it", () => {
    window.localStorage.setItem(
      KEYBINDS_STORAGE_KEY,
      JSON.stringify({ "fixture/broken": 5, "fixture/reload": ["mod+r"] })
    );

    const { service } = mockInjectedService(KeymapService);

    expect(service.getChords(RELOAD)).toEqual(["mod+r"]);
  });

  it("discards an override containing an invalid chord without losing a valid neighbor", () => {
    window.localStorage.setItem(
      KEYBINDS_STORAGE_KEY,
      JSON.stringify({
        [FOCUS_SEARCH_KEYBIND_COMMAND.id]: ["mod+j", "mod"],
        [RELOAD.id]: ["mod+r"],
      })
    );

    const { service } = mockInjectedService(KeymapService);

    expect(service.getChords(FOCUS_SEARCH_KEYBIND_COMMAND)).toEqual(FOCUS_SEARCH_KEYBIND_COMMAND.chords);
    expect(service.getChords(RELOAD)).toEqual(["mod+r"]);
    expect(() => service.keymap).not.toThrow();
  });

  it("keeps an empty stored override as a disabled command", () => {
    window.localStorage.setItem(KEYBINDS_STORAGE_KEY, JSON.stringify({ [FOCUS_SEARCH_KEYBIND_COMMAND.id]: [] }));

    const { service } = mockInjectedService(KeymapService);
    const keymap: IKeymap = service.keymap;

    expect(service.getChords(FOCUS_SEARCH_KEYBIND_COMMAND)).toEqual([]);
    expect(
      toIds([...keymap.anywhere, ...keymap.outsideTextEntry].map((binding: IKeybinding) => binding.command))
    ).not.toContain(FOCUS_SEARCH_KEYBIND_COMMAND.id);
  });

  it("falls back to declarations when the whole map is unreadable", () => {
    window.localStorage.setItem(KEYBINDS_STORAGE_KEY, "{ not json");

    const { service } = mockInjectedService(KeymapService);

    expect(service.getChords(RELOAD)).toEqual(["F5"]);
  });

  it("keeps an override for a command that no longer exists rather than pruning it", () => {
    const stored: string = JSON.stringify({ "fixture/retired": ["mod+r"] });

    window.localStorage.setItem(KEYBINDS_STORAGE_KEY, stored);

    const { service } = mockInjectedService(KeymapService);

    service.setChords(RELOAD.id, ["mod+shift+r"]);

    expect(JSON.parse(window.localStorage.getItem(KEYBINDS_STORAGE_KEY) as string)).toEqual({
      "fixture/reload": ["mod+shift+r"],
      "fixture/retired": ["mod+r"],
    });
  });

  it("restores the declared chords and removes an emptied overlay", () => {
    const { service } = mockInjectedService(KeymapService);

    service.setChords(RELOAD.id, ["mod+r"]);

    expect(service.getChords(RELOAD)).toEqual(["mod+r"]);

    service.setChords(RELOAD.id, null);

    expect(service.getChords(RELOAD)).toEqual(["F5"]);
    expect(window.localStorage.getItem(KEYBINDS_STORAGE_KEY)).toBeNull();
  });

  it("reaches the home screen's commands while nothing is routed", () => {
    const { service } = mockInjectedService(KeymapService);

    expect(toIds(service.commands)).toEqual([...toIds(ROOT_KEYBIND_COMMANDS), ...toIds(LAUNCHER_KEYBIND_COMMANDS)]);
  });

  it("leaves an application that declares none with the root set, not the home screen's", () => {
    const { service } = mockInjectedService(KeymapService);

    service.setApplication({ ...SPRITE_EQUIPMENT_EDITOR_APPLICATION, keybindCommands: undefined });

    expect(toIds(service.commands)).toEqual(toIds(ROOT_KEYBIND_COMMANDS));
    expect(toIds(service.commands)).not.toContain(FOCUS_SEARCH_KEYBIND_COMMAND.id);
  });

  it("adds the routed application's declarations, and drops them when it closes", () => {
    const { service } = mockInjectedService(KeymapService);

    service.setApplication(SPRITE_EQUIPMENT_EDITOR_APPLICATION);

    // The home screen's own declarations go with it: a routed application replaces them rather than adding to them.
    expect(toIds(service.commands)).toEqual([...toIds(ROOT_KEYBIND_COMMANDS), RELOAD_ID]);

    service.setApplication(null);

    expect(toIds(service.commands)).not.toContain(RELOAD_ID);
  });

  it("folds a command listed twice into one, because an id is one command", () => {
    const { service } = mockInjectedService(KeymapService);

    service.setApplication({
      ...SPRITE_EQUIPMENT_EDITOR_APPLICATION,
      keybindCommands: [...ROOT_KEYBIND_COMMANDS, RELOAD_EQUIPMENT_SPRITE_KEYBIND_COMMAND],
    });

    expect(toIds(service.commands)).toEqual([...toIds(ROOT_KEYBIND_COMMANDS), RELOAD_ID]);
  });

  it("builds a keymap through the overrides it holds, and rebuilds it when one changes", () => {
    const { service } = mockInjectedService(KeymapService);

    service.setApplication(SPRITE_EQUIPMENT_EDITOR_APPLICATION);

    const declared: IKeymap = service.keymap;

    expect(toIds(declared.anywhere.map((binding: IKeybinding) => binding.command))).toContain(RELOAD_ID);

    service.setChords(RELOAD_ID, ["/"]);

    // The keymap is derived, so a rebinding moves the command across the suppression split with nothing to invalidate.
    expect(service.keymap).not.toBe(declared);
    expect(toIds(service.keymap.outsideTextEntry.map((binding: IKeybinding) => binding.command))).toContain(RELOAD_ID);
  });
});
