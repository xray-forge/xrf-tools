import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { ContentCopyIcon } from "#/icons/content-copy";
import { SaveIcon } from "#/icons/save";
import { withFrameStrip } from "#/testing/stories";

import { Menubar } from "./Menubar";
import { IMenubarMenu } from "./menubar-menu";

const MENUS: ReadonlyArray<IMenubarMenu> = [
  {
    id: "file",
    label: "File",
    getSections: () => [
      [
        { id: "save", label: "Save", icon: <SaveIcon />, shortcut: "Ctrl+S", onSelect: fn() },
        { id: "save-as", label: "Save as…", shortcut: "Ctrl+Shift+S", isDisabled: true, onSelect: fn() },
      ],
      [
        {
          id: "recent",
          label: "Open recent",
          sections: [
            [
              { id: "recent-1", label: "config\\system.ltx", onSelect: fn() },
              { id: "recent-2", label: "config\\creatures\\actor.ltx", onSelect: fn() },
            ],
          ],
        },
      ],
    ],
  },
  {
    id: "edit",
    label: "Edit",
    getSections: () => [
      [{ id: "copy-path", label: "Copy path", icon: <ContentCopyIcon />, shortcut: "Ctrl+Alt+C", onSelect: fn() }],
    ],
  },
  {
    id: "view",
    label: "View",
    getSections: () => [
      [
        { id: "grid", label: "Grid", description: "Draw the ground grid", isChecked: true, onSelect: fn() },
        { id: "axes", label: "Axes", isChecked: false, onSelect: fn() },
      ],
      [
        { id: "shaded", label: "Shaded", isChecked: true, isExclusive: true, onSelect: fn() },
        { id: "wireframe", label: "Wireframe", isChecked: false, isExclusive: true, onSelect: fn() },
      ],
    ],
  },
];

const meta = {
  title: "Components/Shell/Menubar",
  component: Menubar,
  decorators: [withFrameStrip],
  args: {
    label: "Application menu",
    menus: MENUS,
    onCollapse: fn(),
  },
} satisfies Meta<typeof Menubar>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Shown from the keyboard: the first menu marked, none open. Arrows and Enter walk it. */
export const Closed: TStory = {};

/** Shown by a click: the first menu open, with a disabled row, a submenu, switches and an exclusive pair. */
export const Open: TStory = {
  args: { isFirstMenuOpen: true },
};
