import type { Meta, StoryObj } from "@storybook/react-vite";

import { withFrameStrip } from "#/testing/stories";

import { XrfMark } from "./XrfMark";

const meta = {
  title: "Components/Brand/XrfMark",
  component: XrfMark,
  decorators: [withFrameStrip],
  argTypes: {
    core: { control: "inline-radio", options: ["optimized", "development", "local"] },
    size: { control: { type: "range", min: 12, max: 128, step: 4 } },
  },
  args: {
    core: "optimized",
    size: 16,
  },
} satisfies Meta<typeof XrfMark>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A release build's core, at the caption's size. */
export const Optimized: TStory = {};

/** A continuous-integration build. */
export const Development: TStory = {
  args: { core: "development" },
};

/** A developer machine's own build. */
export const Local: TStory = {
  args: { core: "local" },
};

/** Larger, and named for assistive technology where no label sits beside it. */
export const Titled: TStory = {
  args: { size: 96, title: "xrf" },
};
