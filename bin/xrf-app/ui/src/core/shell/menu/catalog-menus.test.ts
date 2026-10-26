import { describe, expect, it } from "@jest/globals";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { IApplicationDescriptor } from "@/core/routing/application";
import { IMenu, IMenuExtension } from "@/core/shell/menu/lib";
import { FILE_MENU, HELP_MENU } from "@/core/shell/menu/menus";

describe("the menu catalog", () => {
  it("declares each menu id once, so two menus cannot be mistaken for one when entries are laid out", () => {
    const ids: Array<string> = [
      FILE_MENU.id,
      HELP_MENU.id,
      ...APPLICATION_CATALOG.applications.flatMap((it: IApplicationDescriptor) =>
        (it.menus ?? []).map((menu: IMenu) => menu.id)
      ),
    ];

    expect(ids.filter((id: string, index: number) => ids.indexOf(id) !== index)).toEqual([]);
  });

  it("names every application menu after the application declaring it, so an id greps back to its owner", () => {
    const strays: Array<string> = APPLICATION_CATALOG.applications.flatMap((it: IApplicationDescriptor) =>
      (it.menus ?? []).map((menu: IMenu) => menu.id).filter((id: string) => !id.startsWith(`${it.id}/`))
    );

    expect(strays).toEqual([]);
  });

  it("lets an application add sections only to File and Help, under its own name", () => {
    const coreMenuIds: Array<string> = [FILE_MENU.id, HELP_MENU.id];
    const strays: Array<string> = APPLICATION_CATALOG.applications.flatMap((it: IApplicationDescriptor) =>
      (it.menuExtensions ?? [])
        .filter((extension: IMenuExtension) => extension.owner !== it.id || !coreMenuIds.includes(extension.menuId))
        .map((extension: IMenuExtension) => `${it.id} -> ${extension.menuId} as ${extension.owner}`)
    );

    expect(strays).toEqual([]);
  });
});
