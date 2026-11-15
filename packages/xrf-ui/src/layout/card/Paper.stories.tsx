import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { withTileSlot } from "#/testing/stories";
import { Typography } from "#/typography";

import { Paper } from ".";

const PAPER_BODY: ReactElement = (
  <Typography className={"p-4"} variant={"body2"}>
    The overlay level: what floats over the window, its edge carrying the separation in dark.
  </Typography>
);

const meta = {
  title: "Components/Layout/Paper",
  component: Paper,
  decorators: [withTileSlot],
  argTypes: {
    variant: { control: "inline-radio", options: ["elevation", "outlined"] },
    elevation: { control: { type: "range", min: 0, max: 24 } },
  },
  args: {
    elevation: 8,
    children: PAPER_BODY,
  },
} satisfies Meta<typeof Paper>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Flat, with no dark-mode tint: elevations 1 to 4 take the raised shadow, everything above the overlay one. */
export const Elevated: TStory = {};

export const Raised: TStory = {
  args: { elevation: 2 },
};

export const Outlined: TStory = {
  args: { variant: "outlined" },
};
