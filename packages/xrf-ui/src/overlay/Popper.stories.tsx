import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, ReactElement } from "react";

import { Paper } from "#/layout/card";
import { Typography } from "#/typography";

import { Popper } from ".";

type TPopperProps = ComponentProps<typeof Popper>;

/** A point on the page to float from, so the story needs no element of its own to anchor to. */
const VIRTUAL_ANCHOR: NonNullable<TPopperProps["anchorEl"]> = {
  getBoundingClientRect: () => new DOMRect(160, 64, 0, 0),
};

const POPPER_BODY: ReactElement = (
  <Paper className={"w-64 p-3"}>
    <Typography variant={"body2"}>Positioned beside its anchor, with no backdrop and no focus trap.</Typography>
  </Paper>
);

const meta = {
  title: "Components/Overlay/Popper",
  component: Popper,
  argTypes: {
    placement: { control: "select", options: ["bottom-start", "bottom", "right-start", "top-start"] },
  },
  args: {
    open: true,
    anchorEl: VIRTUAL_ANCHOR,
    placement: "bottom-start",
    children: POPPER_BODY,
  },
} satisfies Meta<typeof Popper>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Open: TStory = {};
