import { LEVEL_OBJECT_KINDS, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

import { TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND } from "@/core/level/commands";
import { ILevelHelperOptions, isLevelHelperOn } from "@/core/level/lib/objects/level-helper-options";
import { getLevelVisibleOption, TLevelVisibleOption } from "@/core/level/lib/objects/level-object-visibility";
import { ILevelOverlay, LEVEL_OVERLAYS } from "@/core/level/lib/view/level-overlays";
import { ELevelShading, ILevelShadingChoice } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { EMenuEntryKind, IMenuSubmenuEntry, IMenuToggleEntry, TMenuEntry } from "@/core/shell/menu/lib";

/** What the View menu is drawn from: the view as it stands and the shadings the frame offers now. */
export interface ILevelViewerViewMenuState {
  options: ILevelViewOptions;
  helpers: ILevelHelperOptions;
  shading: ELevelShading;
  /** The shadings offered, FSR's own only while FSR draws the frame. */
  shadings: ReadonlyArray<ILevelShadingChoice>;
  /** Whether the settings draw any parallax for the Parallax switch to turn. */
  isParallaxDrawn: boolean;
}

/** A heading the shadings are listed under. */
type TLevelShadingSection = ILevelShadingChoice["section"];

/** The shadings listed in the Shading submenu itself; the targets the frame is built from each get a further one. */
const FRAME_SHADING_SECTION: TLevelShadingSection = "Frame";

/**
 * @param state - The view as it stands.
 * @param viewService - What the entries switch.
 * @returns The frame's entries: Shading and Overlays.
 */
export function toLevelViewerFrameEntries(
  state: ILevelViewerViewMenuState,
  viewService: LevelViewService
): Array<TMenuEntry> {
  return [toShadingSubmenu(state, viewService), toOverlaysSubmenu(state, viewService)];
}

/**
 * @param state - The view as it stands.
 * @param viewService - What the entries switch.
 * @returns The level objects' entries: Helpers and Objects.
 */
export function toLevelViewerObjectEntries(
  state: ILevelViewerViewMenuState,
  viewService: LevelViewService
): Array<TMenuEntry> {
  return [toHelpersSubmenu(state, viewService), toObjectsSubmenu(state, viewService)];
}

/**
 * What the frame shows, one choice: the frame's own looks first, a further submenu for each kind of target it is built
 * from, then the surfaces' switches, as the toolbar's Shading popover lists them.
 */
function toShadingSubmenu(state: ILevelViewerViewMenuState, viewService: LevelViewService): IMenuSubmenuEntry {
  const targetSections: Array<TLevelShadingSection> = [
    ...new Set(state.shadings.map((choice: ILevelShadingChoice) => choice.section)),
  ].filter((section: TLevelShadingSection) => section !== FRAME_SHADING_SECTION);

  return {
    kind: EMenuEntryKind.SUBMENU,
    id: "level-viewer/shading",
    label: "Shading",
    sections: [
      toShadingChoices(state, viewService, FRAME_SHADING_SECTION),
      targetSections.map((section: TLevelShadingSection): IMenuSubmenuEntry => ({
        kind: EMenuEntryKind.SUBMENU,
        id: `level-viewer/shading-${toKebabCase(section)}`,
        label: section,
        sections: [toShadingChoices(state, viewService, section)],
      })),
      [
        toOptionToggle(state, viewService, { option: "isWireframe", label: "Wireframe" }),
        toOptionToggle(state, viewService, { option: "isBumped", label: "Bumps" }),
        {
          ...toOptionToggle(state, viewService, { option: "isParallaxed", label: "Parallax" }),
          isChecked: state.options.isParallaxed && state.isParallaxDrawn,
          isDisabled: !state.isParallaxDrawn,
        },
        toOptionToggle(state, viewService, { option: "isWallmarked", label: "Wall marks" }),
      ],
    ],
  };
}

/** The shadings offered under one heading, each a choice, the one shown chosen. */
function toShadingChoices(
  state: ILevelViewerViewMenuState,
  viewService: LevelViewService,
  section: TLevelShadingSection
): Array<TMenuEntry> {
  return state.shadings
    .filter((choice: ILevelShadingChoice) => choice.section === section)
    .map((choice: ILevelShadingChoice): TMenuEntry => ({
      kind: EMenuEntryKind.CHOICE,
      id: `level-viewer/shade-${toKebabCase(choice.value)}`,
      label: choice.label,
      isChosen: choice.value === state.shading,
      onChoose: () => viewService.setShading(choice.value),
    }));
}

/** What is drawn over the frame, as the toolbar's Overlays popover lists it. */
function toOverlaysSubmenu(state: ILevelViewerViewMenuState, viewService: LevelViewService): IMenuSubmenuEntry {
  return {
    kind: EMenuEntryKind.SUBMENU,
    id: "level-viewer/overlays",
    label: "Overlays",
    sections: [
      LEVEL_OVERLAYS.map((overlay: ILevelOverlay): TMenuEntry => ({
        ...toOptionToggle(state, viewService, { option: overlay.option, label: overlay.label }),
        id: `level-viewer/toggle-${overlay.id}`,
      })),
    ],
  };
}

/** The helpers' master switch and how they are drawn, then each kind's helper, which waits on the master switch. */
function toHelpersSubmenu(state: ILevelViewerViewMenuState, viewService: LevelViewService): IMenuSubmenuEntry {
  const { helpers } = state;

  return {
    kind: EMenuEntryKind.SUBMENU,
    id: "level-viewer/helpers",
    label: "Helpers",
    sections: [
      [
        {
          kind: EMenuEntryKind.TOGGLE,
          id: "level-viewer/toggle-helpers",
          label: "Show helpers",
          command: TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND,
          isChecked: helpers.isShown,
          onToggle: viewService.toggleHelpers,
        },
        {
          kind: EMenuEntryKind.TOGGLE,
          id: "level-viewer/toggle-rest-faded",
          label: "Fade the rest",
          isChecked: helpers.isRestFaded,
          onToggle: () => viewService.setHelperLook({ isRestFaded: !helpers.isRestFaded }),
        },
        {
          kind: EMenuEntryKind.TOGGLE,
          id: "level-viewer/toggle-screenshot-helped",
          label: "Include in screenshots",
          isChecked: helpers.isScreenshotHelped,
          onToggle: () => viewService.setHelperLook({ isScreenshotHelped: !helpers.isScreenshotHelped }),
        },
      ],
      LEVEL_OBJECT_KINDS.map((entry: LevelObjectKindEntry): TMenuEntry => {
        const isHelped: boolean = isLevelHelperOn(helpers, entry.kind);

        return {
          kind: EMenuEntryKind.TOGGLE,
          id: `level-viewer/toggle-${toKebabCase(entry.kind)}-helper`,
          label: entry.label,
          isChecked: isHelped,
          isDisabled: !helpers.isShown,
          onToggle: () => viewService.setHelperKinds({ isOn: !isHelped, kinds: [entry.kind] }),
        };
      }),
    ],
  };
}

/** Which kinds of level object are drawn, as the toolbar's Show popover lists them, then what a new game releases. */
function toObjectsSubmenu(state: ILevelViewerViewMenuState, viewService: LevelViewService): IMenuSubmenuEntry {
  return {
    kind: EMenuEntryKind.SUBMENU,
    id: "level-viewer/objects",
    label: "Objects",
    sections: [
      LEVEL_OBJECT_KINDS.flatMap((entry: LevelObjectKindEntry): Array<TMenuEntry> => {
        const option: Nullable<TLevelVisibleOption> = getLevelVisibleOption(entry.kind);

        return option ? [toOptionToggle(state, viewService, { option, label: entry.label })] : [];
      }),
      [toOptionToggle(state, viewService, { option: "isSpawnedReleased", label: "Released" })],
    ],
  };
}

/** One of the view's switches as a menu row. */
function toOptionToggle(
  state: ILevelViewerViewMenuState,
  viewService: LevelViewService,
  { option, label }: { option: keyof ILevelViewOptions; label: string }
): IMenuToggleEntry {
  return {
    kind: EMenuEntryKind.TOGGLE,
    id: `level-viewer/toggle-${toKebabCase(option.replace(/^is/, ""))}`,
    label,
    isChecked: state.options[option],
    onToggle: () => viewService.toggleOption(option),
  };
}

/** A camel or title case name as an entry id's kebab case. */
function toKebabCase(name: string): string {
  return name.replace(/(?<=[a-z0-9])(?=[A-Z])/g, "-").toLowerCase();
}
