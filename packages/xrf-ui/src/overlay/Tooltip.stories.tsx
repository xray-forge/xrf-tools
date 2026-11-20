import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { Button } from "#/button";
import { withOverlayRoom } from "#/testing/stories";

import { Tooltip } from ".";

const TOOLTIP_TARGET: ReactElement = (
  <Button variant={"outlined"} size={"small"} onClick={fn()}>
    Reload
  </Button>
);

const meta = {
  title: "Components/Overlay/Tooltip",
  component: Tooltip,
  decorators: [withOverlayRoom],
  argTypes: {
    placement: { control: "select", options: ["top", "bottom", "left", "right", "top-start", "bottom-start"] },
  },
  args: {
    title: "Read the archives again",
    open: true,
    children: TOOLTIP_TARGET,
  },
} satisfies Meta<typeof Tooltip>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** On the overlay level rather than MUI's dark grey, after a 400 ms delay when not held open. */
export const Open: TStory = {};

/** Hover the button: the theme's enter delay applies. */
export const OnHover: TStory = {
  args: { open: undefined },
};
