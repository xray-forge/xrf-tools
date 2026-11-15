import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { ArchiveIcon } from "#/icons/archive";
import { DescriptionOutlinedIcon } from "#/icons/description-outlined";
import { SettingsIcon } from "#/icons/settings";
import { withPanelSlot } from "#/testing/stories";

import { List, ListItem, ListItemButton, ListItemIcon, ListItemText, ListSubheader } from ".";

const TOOL_ROWS: ReactElement = (
  <>
    <ListSubheader className={"bg-transparent"}>Tools</ListSubheader>
    <ListItemButton selected onClick={fn()}>
      <ListItemIcon>
        <ArchiveIcon fontSize={"small"} />
      </ListItemIcon>
      <ListItemText primary={"Archives"} secondary={"Unpack and pack volumes"} />
    </ListItemButton>
    <ListItemButton onClick={fn()}>
      <ListItemIcon>
        <DescriptionOutlinedIcon fontSize={"small"} />
      </ListItemIcon>
      <ListItemText primary={"Configs"} />
    </ListItemButton>
    <ListItemButton disabled onClick={fn()}>
      <ListItemIcon>
        <SettingsIcon fontSize={"small"} />
      </ListItemIcon>
      <ListItemText primary={"Settings"} secondary={"Open a game first"} />
    </ListItemButton>
    <ListItem>
      <ListItemText primary={"A plain row"} />
    </ListItem>
  </>
);

const meta = {
  title: "Components/List/List",
  component: List,
  decorators: [withPanelSlot],
  args: {
    dense: true,
    children: TOOL_ROWS,
  },
} satisfies Meta<typeof List>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A chosen row takes the shared `selected` scrim rather than MUI's accent tint, and its text sizes are the theme's. */
export const Rows: TStory = {};
