import { describe, expect, it } from "@jest/globals";

import { IToolDescriptor } from "@/core/routing/tool";
import { IMenu, IMenuExtension } from "@/core/shell/menu/lib";
import { FILE_MENU, HELP_MENU } from "@/core/shell/menu/menus";
import { TOOL_CATALOG } from "@/ToolCatalog";

describe("the menu catalog", () => {
  it("declares each menu id once, so two menus cannot be mistaken for one when entries are laid out", () => {
    const ids: Array<string> = [
      FILE_MENU.id,
      HELP_MENU.id,
      ...TOOL_CATALOG.tools.flatMap((it: IToolDescriptor) => (it.menus ?? []).map((menu: IMenu) => menu.id)),
    ];

    expect(ids.filter((id: string, index: number) => ids.indexOf(id) !== index)).toEqual([]);
  });

  it("names every tool menu after the tool declaring it, so an id greps back to its owner", () => {
    const strays: Array<string> = TOOL_CATALOG.tools.flatMap((it: IToolDescriptor) =>
      (it.menus ?? []).map((menu: IMenu) => menu.id).filter((id: string) => !id.startsWith(`${it.id}/`))
    );

    expect(strays).toEqual([]);
  });

  it("lets a tool add sections only to File and Help, under its own name", () => {
    const coreMenuIds: Array<string> = [FILE_MENU.id, HELP_MENU.id];
    const strays: Array<string> = TOOL_CATALOG.tools.flatMap((it: IToolDescriptor) =>
      (it.menuExtensions ?? [])
        .filter((extension: IMenuExtension) => extension.owner !== it.id || !coreMenuIds.includes(extension.menuId))
        .map((extension: IMenuExtension) => `${it.id} -> ${extension.menuId} as ${extension.owner}`)
    );

    expect(strays).toEqual([]);
  });
});
