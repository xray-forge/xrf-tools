import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { ArchiveIcon } from "#/icons/archive";
import { BarChartIcon } from "#/icons/bar-chart";
import { FolderOpenIcon } from "#/icons/folder-open";
import { SettingsIcon } from "#/icons/settings";
import { UnarchiveIcon } from "#/icons/unarchive";
import { withPanelSlot } from "#/testing/stories";
import { Typography } from "#/typography";

import { EditorSideMenu, IEditorSideMenuItem } from "./EditorSideMenu";

const SECTIONS: Array<IEditorSideMenuItem> = [
  {
    label: "Archives",
    description: "12 volumes, 38,113 files",
    icon: <ArchiveIcon />,
    isSelected: true,
    onClick: fn(),
  },
  { label: "Statistics", description: "Sizes by extension", icon: <BarChartIcon />, onClick: fn() },
  { label: "Settings", description: "Unpack and pack options", icon: <SettingsIcon />, onClick: fn() },
];

const ACTIONS: Array<IEditorSideMenuItem> = [
  { label: "Open folder", icon: <FolderOpenIcon />, onClick: fn() },
  {
    label: "Unpack all",
    description: "Nothing is loaded yet",
    icon: <UnarchiveIcon />,
    isDisabled: true,
    onClick: fn(),
  },
];

const HEADER: ReactElement = (
  <Typography className={"px-4 pt-3 pb-2 text-text-secondary"} variant={"overline"}>
    gamedata
  </Typography>
);

const FOOTER: ReactElement = (
  <Typography className={"block px-4 py-2 text-text-secondary"} variant={"caption"}>
    Last unpacked 14:02, 38,113 files
  </Typography>
);

const meta = {
  title: "Components/Editor/EditorSideMenu",
  component: EditorSideMenu,
  decorators: [withPanelSlot],
  argTypes: {
    sections: { control: false },
    actions: { control: false },
  },
  args: {
    header: HEADER,
    sections: SECTIONS,
    actions: ACTIONS,
  },
} satisfies Meta<typeof EditorSideMenu>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Sections in the scrolling middle, actions pinned under a rule at the bottom. */
export const Sections: TStory = {};

/** What the last action has to report, pinned directly above the actions. */
export const WithFooter: TStory = {
  args: { footer: FOOTER },
};

/** Arbitrary content in place of the section list. */
export const WithContent: TStory = {
  args: {
    sections: undefined,
    children: (
      <Typography className={"px-4 py-2 text-text-secondary"} variant={"body2"}>
        Open a game folder to list its archives.
      </Typography>
    ),
  },
};
