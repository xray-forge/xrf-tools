import { describe, expect, it } from "@jest/globals";

import { defineKeybindCommand, EKeybindCommandCategory, IKeybindCommand } from "@/core/commands";

import { buildKeymap, findChordConflicts, IKeymap, resolveKeybinding } from "./keymap";

const SEARCH: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.NAVIGATION,
  chords: ["mod+k", "/"],
  description: "Search.",
  id: "fixture/search",
  label: "Search",
});

const RELOAD: IKeybindCommand = defineKeybindCommand({
  category: EKeybindCommandCategory.APPLICATION,
  chords: ["F5"],
  description: "Reload.",
  id: "fixture/reload",
  label: "Reload",
});

function mockKeyEvent(init: KeyboardEventInit): KeyboardEvent {
  return new KeyboardEvent("keydown", { altKey: false, ctrlKey: false, metaKey: false, shiftKey: false, ...init });
}

function getKeymapOf(commands: ReadonlyArray<IKeybindCommand>): IKeymap {
  return buildKeymap(commands, (command: IKeybindCommand) => command.chords);
}

describe("buildKeymap", () => {
  it("partitions by suppression so typing scans only what could fire in a field", () => {
    const keymap: IKeymap = getKeymapOf([SEARCH, RELOAD]);

    expect(keymap.anywhere.map((binding) => binding.command.id)).toEqual(["fixture/search", "fixture/reload"]);
    expect(keymap.outsideTextEntry.map((binding) => binding.command.id)).toEqual(["fixture/search"]);
  });

  it("binds a command listed twice once, because a composed core set is the same command", () => {
    const keymap: IKeymap = getKeymapOf([SEARCH, RELOAD, SEARCH]);

    expect(keymap.anywhere).toHaveLength(2);
    expect(keymap.outsideTextEntry).toHaveLength(1);
  });

  it("takes chords from the resolver, so an override replaces the declared one", () => {
    const keymap: IKeymap = buildKeymap([RELOAD], () => ["mod+r"]);

    expect(resolveKeybinding(keymap, mockKeyEvent({ key: "F5" }), false)).toBeNull();
    expect(resolveKeybinding(keymap, mockKeyEvent({ ctrlKey: true, key: "r" }), false)).toBe(RELOAD);
  });
});

describe("resolveKeybinding", () => {
  it("finds a command by any of its chords", () => {
    const keymap: IKeymap = getKeymapOf([SEARCH, RELOAD]);

    expect(resolveKeybinding(keymap, mockKeyEvent({ ctrlKey: true, key: "k" }), false)).toBe(SEARCH);
    expect(resolveKeybinding(keymap, mockKeyEvent({ key: "/" }), false)).toBe(SEARCH);
    expect(resolveKeybinding(keymap, mockKeyEvent({ key: "F5" }), false)).toBe(RELOAD);
  });

  it("withholds a bare chord from text entry while keeping the modified one", () => {
    const keymap: IKeymap = getKeymapOf([SEARCH]);

    expect(resolveKeybinding(keymap, mockKeyEvent({ key: "/" }), true)).toBeNull();
    expect(resolveKeybinding(keymap, mockKeyEvent({ ctrlKey: true, key: "k" }), true)).toBe(SEARCH);
  });

  it("answers nothing for an unbound key", () => {
    expect(resolveKeybinding(getKeymapOf([SEARCH]), mockKeyEvent({ key: "q" }), false)).toBeNull();
  });
});

describe("findChordConflicts", () => {
  it("reports a chord two commands claim, in canonical spelling", () => {
    const other: IKeybindCommand = defineKeybindCommand({
      category: EKeybindCommandCategory.VIEW,
      chords: ["MOD + K"],
      description: "Other.",
      id: "fixture/other",
      label: "Other",
    });

    expect(findChordConflicts([SEARCH, other])).toEqual([
      { chord: "mod+k", commandIds: ["fixture/search", "fixture/other"] },
    ]);
  });

  it("is quiet when chords are distinct, including one command listed twice", () => {
    expect(findChordConflicts([SEARCH, RELOAD, SEARCH])).toEqual([]);
  });
});
