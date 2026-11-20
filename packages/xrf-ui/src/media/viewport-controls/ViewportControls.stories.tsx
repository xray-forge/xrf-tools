import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withViewportPane } from "#/testing/stories";

import { ViewportControls } from "./ViewportControls";

const meta = {
  title: "Components/Media/ViewportControls",
  component: ViewportControls,
  decorators: [withViewportPane],
  args: {
    zoom: { scale: 2, onActualSize: fn() },
    onZoomIn: fn(),
    onZoomOut: fn(),
    onReset: fn(),
  },
} satisfies Meta<typeof ViewportControls>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A picture's bar: the magnification between the zoom steps, a click on it back to one to one. */
export const Picture: TStory = {};

/** A scene's bar, where one to one means nothing. */
export const Scene: TStory = {
  args: { zoom: undefined },
};
