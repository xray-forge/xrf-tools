import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { Box, Divider, Stack } from ".";

const STACK_ITEMS: ReactElement = (
  <>
    {["configs.db", "resources.db", "sounds.db"].map((name: string) => (
      <Box key={name} className={"rounded-control border border-divider bg-well px-3 py-2 font-monospace text-xs"}>
        {name}
      </Box>
    ))}
  </>
);

const meta = {
  title: "Components/Layout/Stack",
  component: Stack,
  argTypes: {
    direction: { control: "inline-radio", options: ["column", "row"] },
    spacing: { control: { type: "range", min: 0, max: 4, step: 0.5 } },
  },
  args: {
    direction: "column",
    spacing: 1,
    children: STACK_ITEMS,
  },
} satisfies Meta<typeof Stack>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** `Box`, `Stack` and `Divider` together: spacing in theme units, children drawn as boxes. */
export const Column: TStory = {};

export const RowWithDividers: TStory = {
  args: { direction: "row", divider: <Divider orientation={"vertical"} flexItem /> },
};
