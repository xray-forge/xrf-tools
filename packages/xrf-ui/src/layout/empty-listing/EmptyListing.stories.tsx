import type { Meta, StoryObj } from "@storybook/react-vite";

import { withPanelSlot } from "#/testing/stories";

import { EmptyListing } from "./EmptyListing";

const meta = {
  title: "Components/Layout/EmptyListing",
  component: EmptyListing,
  decorators: [withPanelSlot],
  args: {
    label: "No texture matches 'bandit_mask'.",
  },
} satisfies Meta<typeof EmptyListing>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Stands where the rows of a searchable listing would be. */
export const Default: TStory = {};
