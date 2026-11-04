import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";

import { TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND } from "@/core/level/commands";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { ELevelShading, LEVEL_SHADINGS } from "@/core/level/lib/view/level-shading";
import { LevelViewService } from "@/core/level/services/level-view";
import { EMenuEntryKind, IMenuSubmenuEntry, TMenuEntry } from "@/core/shell/menu/lib";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewerViewMenuState,
  toLevelViewerFrameEntries,
  toLevelViewerObjectEntries,
} from "@/tools/level-viewer/components/menus/LevelViewerViewMenu.utils";

function toState(
  view: LevelViewService,
  overrides: Partial<ILevelViewerViewMenuState> = {}
): ILevelViewerViewMenuState {
  return {
    options: view.options,
    helpers: view.helpers,
    shading: ELevelShading.FINAL,
    shadings: LEVEL_SHADINGS,
    isParallaxDrawn: true,
    ...overrides,
  };
}

function toSubmenu(entries: ReadonlyArray<TMenuEntry>, label: string): IMenuSubmenuEntry {
  const found: TMenuEntry | undefined = entries.find(
    (it: TMenuEntry) => it.kind === EMenuEntryKind.SUBMENU && it.label === label
  );

  if (found?.kind !== EMenuEntryKind.SUBMENU) {
    throw new Error(`No submenu '${label}'.`);
  }

  return found;
}

function toLabels(entries: ReadonlyArray<TMenuEntry>): Array<string> {
  return entries.map((it: TMenuEntry) => (it.kind === EMenuEntryKind.COMMAND ? it.command.label : it.label));
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelViewerViewMenu.utils", () => {
  it("lists the frame's looks as choices, and each kind of target as a further submenu", () => {
    const view: LevelViewService = mockLevelViewContainer().view;
    const shading: IMenuSubmenuEntry = toSubmenu(toLevelViewerFrameEntries(toState(view), view), "Shading");
    const [looks, targets] = shading.sections;

    expect(toLabels(looks)).toEqual(["Final frame", "Clay", "Shader colours", "Overdraw"]);
    expect(looks.map((it: TMenuEntry) => it.kind === EMenuEntryKind.CHOICE && it.isChosen)).toEqual([
      true,
      false,
      false,
      false,
    ]);
    expect(toLabels(targets)).toEqual(["Surfaces", "Light", "Temporal"]);
    expect(toLabels(toSubmenu(targets, "Surfaces").sections[0])).toContain("Albedo");
    // The Shading submenu is the only place a target is offered, so each carries how to read it.
    expect(looks[0]).toEqual(expect.objectContaining({ description: "The finished frame, every pass applied." }));
  });

  it("chooses a shading through the view", () => {
    const view: LevelViewService = mockLevelViewContainer().view;
    const [looks] = toSubmenu(toLevelViewerFrameEntries(toState(view), view), "Shading").sections;
    const clay: TMenuEntry = looks[1];

    act(() => {
      if (clay.kind === EMenuEntryKind.CHOICE) {
        clay.onChoose();
      }
    });

    expect(view.shading).toBe(ELevelShading.CLAY);
  });

  it("shows Parallax disabled and off while the settings draw none", () => {
    const view: LevelViewService = mockLevelViewContainer().view;
    const [, , switches] = toSubmenu(
      toLevelViewerFrameEntries(toState(view, { isParallaxDrawn: false }), view),
      "Shading"
    ).sections;

    expect(switches.find((it: TMenuEntry) => it.kind === EMenuEntryKind.TOGGLE && it.label === "Parallax")).toEqual(
      expect.objectContaining({ isChecked: false, isDisabled: true })
    );
  });

  it("offers each kind's helper disabled until helpers are shown, the master switch carrying its chord", () => {
    const view: LevelViewService = mockLevelViewContainer().view;
    const [look, kinds] = toSubmenu(toLevelViewerObjectEntries(toState(view), view), "Helpers").sections;

    expect(look[0]).toEqual(
      expect.objectContaining({
        label: "Show helpers",
        command: TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND,
        isChecked: false,
      })
    );
    expect(kinds.every((it: TMenuEntry) => it.kind === EMenuEntryKind.TOGGLE && it.isDisabled)).toBe(true);

    act(() => view.toggleHelpers());

    const [, shownKinds] = toSubmenu(toLevelViewerObjectEntries(toState(view), view), "Helpers").sections;

    expect(shownKinds.some((it: TMenuEntry) => it.kind === EMenuEntryKind.TOGGLE && it.isDisabled)).toBe(false);
  });

  it("switches the drawn kinds and what a new game releases", () => {
    const view: LevelViewService = mockLevelViewContainer().view;
    const [kinds, released] = toSubmenu(toLevelViewerObjectEntries(toState(view), view), "Objects").sections;

    expect(toLabels(kinds)).toEqual(["Props", "Items", "Weapons", "Lamps"]);
    expect(toLabels(released)).toEqual(["Released"]);

    act(() => {
      if (kinds[1].kind === EMenuEntryKind.TOGGLE) {
        kinds[1].onToggle();
      }
    });

    expect(view.options.isSpawnedItems).toBe(false);
  });
});
