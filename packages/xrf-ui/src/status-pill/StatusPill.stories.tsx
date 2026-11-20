import type { Meta, StoryObj } from "@storybook/react-vite";

import { withTileSlot } from "#/testing/stories";
import { Typography } from "#/typography";

import { StatusPill } from "./StatusPill";

const meta = {
  title: "Components/StatusPill/StatusPill",
  component: StatusPill,
  args: {
    label: "Planned",
  },
} satisfies Meta<typeof StatusPill>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Planned: TStory = {};

/** Beside a tile's title, as the launcher marks a tool the roster names but does not implement yet. */
export const BesideTitle: TStory = {
  decorators: [
    (Story) => (
      <div className={"flex items-start justify-between gap-2"}>
        <Typography variant={"subtitle2"}>Spawn editor</Typography>
        <Story />
      </div>
    ),
    withTileSlot,
  ],
};
