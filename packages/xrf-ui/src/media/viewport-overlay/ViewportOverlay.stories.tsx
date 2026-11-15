import type { Meta, StoryObj } from "@storybook/react-vite";

import { withViewportPane } from "#/testing/stories";

import { ViewportOverlay } from "./ViewportOverlay";

const meta = {
  title: "Components/Media/ViewportOverlay",
  component: ViewportOverlay,
  decorators: [withViewportPane],
  args: {
    corner: "top-left",
    children: "60 fps · 16.6 ms",
  },
} satisfies Meta<typeof ViewportOverlay>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A frame readout, where the level viewer keeps its timings. */
export const TopLeft: TStory = {};

/** The camera's coordinates, where the level viewer keeps them. */
export const BottomRight: TStory = {
  args: { corner: "bottom-right", children: "x 112.4  y 3.1  z -48.0" },
};

/** Several lines, as a picked object's readout runs. */
export const Lines: TStory = {
  args: {
    corner: "bottom-left",
    children: (
      <>
        <div>esc_stalker_bandit_0003</div>
        <div>stalker · physic_object</div>
      </>
    ),
  },
};
