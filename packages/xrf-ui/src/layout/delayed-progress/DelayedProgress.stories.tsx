import type { Meta, StoryObj } from "@storybook/react-vite";

import { withListingPane, withViewportPane } from "#/testing/stories";

import { DelayedProgress } from "./DelayedProgress";

const meta = {
  title: "Components/Layout/DelayedProgress",
  component: DelayedProgress,
  decorators: [withListingPane],
  args: {
    label: "Reading configs.db",
  },
} satisfies Meta<typeof DelayedProgress>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Revealed only after a delay, so a fast read never flashes it. */
export const Default: TStory = {};

/** In a viewport's colours, over its backdrop. */
export const OnViewport: TStory = {
  decorators: [withViewportPane],
  args: { label: "Loading l01_escape geometry", isOnViewport: true },
};
