import { describe, expect, it, jest } from "@jest/globals";

import { EToolId } from "@/core/routing/tool";

import { assembleMenus, IAssembledMenu } from "./menu-assembly";
import { defineMenu, IMenu } from "./menu-descriptor";
import { IMenuContribution, TMenuEntry } from "./menu-entry";
import { defineMenuExtension, IMenuExtension } from "./menu-extension";

const FIRST: IMenu<"top" | "bottom"> = defineMenu({ id: "test/first", label: "First", sections: ["top", "bottom"] });
const SECOND: IMenu<"only"> = defineMenu({ id: "test/second", label: "Second", sections: ["only"] });
const SLOTTED: IMenu<"top" | "bottom"> = defineMenu({
  id: "test/slotted",
  label: "Slotted",
  sections: ["top", "bottom"],
  extensionsBefore: "bottom",
});

function action(id: string): TMenuEntry {
  return { kind: "action", id, label: id, onSelect: jest.fn() };
}

function toIds(menus: ReadonlyArray<IAssembledMenu>): Array<Array<Array<string>>> {
  return menus.map((it: IAssembledMenu) =>
    it.sections.map((entries: ReadonlyArray<TMenuEntry>) =>
      entries.map((entry: TMenuEntry) => (entry.kind === "command" ? entry.command.id : entry.id))
    )
  );
}

describe("defineMenu", () => {
  it("refuses an id that does not grep back to its owner, and a section named twice", () => {
    expect(() => defineMenu({ id: "File", label: "File", sections: ["a"] })).toThrow(/<domain>\/<noun>/);
    expect(() => defineMenu({ id: "test/menu", label: "Menu", sections: ["a", "a"] })).toThrow(/twice/);
    expect(() => defineMenu({ id: "test/menu", label: "Menu", sections: [] })).toThrow(/no section/);
  });
});

describe("defineMenuExtension", () => {
  it("prefixes each section with its owner, so an added section never meets one the menu declares", () => {
    const extension: IMenuExtension<"top"> = defineMenuExtension(SLOTTED, {
      owner: EToolId.LEVEL_EDITOR,
      sections: ["top"],
    });

    expect(extension.sections.top).toEqual({ id: "level-editor/top", menuId: "test/slotted" });
    expect(() => defineMenuExtension(SLOTTED, { owner: EToolId.LEVEL_EDITOR, sections: [] })).toThrow(/no section/);
  });
});

describe("assembleMenus", () => {
  it("draws the sections a tool adds at the menu's extension point, or after its last without one", () => {
    const open: IMenuExtension<"open" | "compile"> = defineMenuExtension(SLOTTED, {
      owner: EToolId.LEVEL_EDITOR,
      sections: ["open", "compile"],
    });
    const tail: IMenuExtension<"extra"> = defineMenuExtension(FIRST, {
      owner: EToolId.LEVEL_EDITOR,
      sections: ["extra"],
    });
    const contributions: Array<IMenuContribution> = [
      { section: SLOTTED.sections.top, entries: [action("top")] },
      { section: SLOTTED.sections.bottom, entries: [action("bottom")] },
      { section: open.sections.compile, entries: [action("compile")] },
      { section: open.sections.open, entries: [action("open")] },
      { section: FIRST.sections.top, entries: [action("first")] },
      { section: tail.sections.extra, entries: [action("extra")] },
    ];

    expect(toIds(assembleMenus([SLOTTED, FIRST], [open, tail], contributions))).toEqual([
      [["top"], ["open"], ["compile"], ["bottom"]],
      [["first"], ["extra"]],
    ]);
  });

  it("draws sections in declared order and entries in publication order, whatever order they were published in", () => {
    const contributions: Array<IMenuContribution> = [
      { section: FIRST.sections.bottom, entries: [action("c")] },
      { section: FIRST.sections.top, entries: [action("a")] },
      { section: FIRST.sections.top, entries: [action("b")] },
    ];

    expect(toIds(assembleMenus([FIRST], [], contributions))).toEqual([[["a", "b"], ["c"]]]);
  });

  it("drops an empty section and a menu left with nothing, keeping the bar's order", () => {
    const contributions: Array<IMenuContribution> = [
      { section: SECOND.sections.only, entries: [action("x")] },
      { section: FIRST.sections.bottom, entries: [action("y")] },
      { section: FIRST.sections.top, entries: [] },
    ];

    const menus: Array<IAssembledMenu> = assembleMenus([FIRST, SECOND], [], contributions);

    expect(menus.map((it: IAssembledMenu) => it.menu.label)).toEqual(["First", "Second"]);
    expect(toIds(menus)).toEqual([[["y"]], [["x"]]]);
    expect(assembleMenus([FIRST], [], [])).toEqual([]);
  });

  it("ignores entries published into a menu the bar does not show, such as another tool's", () => {
    const contributions: Array<IMenuContribution> = [{ section: SECOND.sections.only, entries: [action("x")] }];

    expect(assembleMenus([FIRST], [], contributions)).toEqual([]);
  });

  it("matches a section by identity, so an equal declaration from another module instance still lands", () => {
    // Jest can load a module twice on Windows, and a reference check would then drop every entry silently.
    const contributions: Array<IMenuContribution> = [
      { section: { id: "top", menuId: "test/first" }, entries: [action("a")] },
    ];

    expect(toIds(assembleMenus([FIRST], [], contributions))).toEqual([[["a"]]]);
  });
});
