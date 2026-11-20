import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { withTileSlot } from "#/testing/stories";
import { Typography } from "#/typography";

import { Card, CardActionArea } from ".";

const CARD_BODY: ReactElement = (
  <div className={"flex flex-col gap-1 p-4"}>
    <Typography variant={"subtitle2"}>Archives</Typography>
    <Typography className={"text-text-secondary"} variant={"body2"}>
      Unpack, pack and patch the game&apos;s volumes.
    </Typography>
  </div>
);

const ACTION_BODY: ReactElement = <CardActionArea onClick={fn()}>{CARD_BODY}</CardActionArea>;

const meta = {
  title: "Components/Layout/Card",
  component: Card,
  decorators: [withTileSlot],
  args: {
    children: CARD_BODY,
  },
} satisfies Meta<typeof Card>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Glass on the frame level with the card-only inset bevels, as the launcher and the picker screens draw it. */
export const Default: TStory = {};

/** With an action area: hovering it thins the glass and turns the border to the accent. */
export const Actionable: TStory = {
  args: { children: ACTION_BODY },
};
