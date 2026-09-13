import { beforeEach, describe, expect, it } from "@jest/globals";

import { SPRITE_EQUIPMENT_EDITOR_APPLICATION } from "@/applications/sprite-equipment-editor";
import { RELOAD_EQUIPMENT_SPRITE_COMMAND } from "@/applications/sprite-equipment-editor/commands";
import { defineCommand, ECommandCategory, ICommandDescriptor } from "@/core/commands";
import { ROOT_COMMANDS } from "@/core/commands/root-commands";
import { IKeybinding, IKeymap } from "@/core/keybinds/lib/keymap";
import { KEYBINDS_STORAGE_KEY } from "@/core/storage";
import { mockInjectedService } from "@/fixtures/utils/container";

import { KeymapService } from "./keymap.service";

const RELOAD_ID: string = RELOAD_EQUIPMENT_SPRITE_COMMAND.id;

function toIds(commands: ReadonlyArray<ICommandDescriptor>): Array<string> {
  return commands.map((command: ICommandDescriptor) => command.id);
}

const RELOAD: ICommandDescriptor = defineCommand({
  category: ECommandCategory.APPLICATION,
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

  it("reaches only the root commands until an application is routed", () => {
    const { service } = mockInjectedService(KeymapService);

    expect(toIds(service.commands)).toEqual(toIds(ROOT_COMMANDS));
  });

  it("adds the routed application's declarations, and drops them when it closes", () => {
    const { service } = mockInjectedService(KeymapService);

    service.setApplication(SPRITE_EQUIPMENT_EDITOR_APPLICATION);

    expect(toIds(service.commands)).toEqual([...toIds(ROOT_COMMANDS), RELOAD_ID]);

    service.setApplication(null);

    expect(toIds(service.commands)).not.toContain(RELOAD_ID);
  });

  it("folds a command listed twice into one, because an id is one command", () => {
    const { service } = mockInjectedService(KeymapService);

    service.setApplication({
      ...SPRITE_EQUIPMENT_EDITOR_APPLICATION,
      commands: [...ROOT_COMMANDS, RELOAD_EQUIPMENT_SPRITE_COMMAND],
    });

    expect(toIds(service.commands)).toEqual([...toIds(ROOT_COMMANDS), RELOAD_ID]);
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
