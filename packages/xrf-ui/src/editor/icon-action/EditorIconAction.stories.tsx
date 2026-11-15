import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { GridOnIcon } from "#/icons/grid-on";
import { SaveIcon } from "#/icons/save";
import { VisibilityIcon } from "#/icons/visibility";
import { withFrameStrip } from "#/testing/stories";

import { EditorIconAction } from "./EditorIconAction";

const GRID_ACTION: ReactElement = (
  <EditorIconAction label={"Grid"} description={"Show the ground grid"} icon={<GridOnIcon />} isActive onClick={fn()} />
);

const SAVE_ACTION: ReactElement = (
  <EditorIconAction label={"Save"} description={"Nothing has changed"} icon={<SaveIcon />} isDisabled onClick={fn()} />
);

const meta = {
  title: "Components/Editor/EditorIconAction",
  component: EditorIconAction,
  args: {
    label: "Grid",
    description: "Show the ground grid",
    icon: <GridOnIcon />,
    onClick: fn(),
  },
} satisfies Meta<typeof EditorIconAction>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Rest: TStory = {};

export const Active: TStory = {
  args: { isActive: true },
};

export const Disabled: TStory = {
  args: { isDisabled: true, description: "Nothing is loaded to draw a grid under" },
};

/** The tooltip heads the description with the title, naming what the action is before what it stands at. */
export const WithTitle: TStory = {
  args: { label: "Save", title: "Save", description: "Write the changes to gamedata", icon: <SaveIcon /> },
};

/** Between an active and a disabled action on the frame, as a toolbar carries them. */
export const Toolbar: TStory = {
  decorators: [
    (Story) => (
      <>
        {GRID_ACTION}
        <Story />
        {SAVE_ACTION}
      </>
    ),
    withFrameStrip,
  ],
  args: { label: "Objects", description: "Show objects", icon: <VisibilityIcon /> },
};
