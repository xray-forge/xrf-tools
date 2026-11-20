import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withFrameStrip } from "#/testing/stories";

import { WindowControls } from "./WindowControls";

const meta = {
  title: "Components/Shell/WindowControls",
  component: WindowControls,
  decorators: [withFrameStrip],
  args: {
    isMaximized: false,
    onMinimize: fn(),
    onToggleMaximize: fn(),
    onClose: fn(),
  },
} satisfies Meta<typeof WindowControls>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Windowed: TStory = {};

/** Maximize turns to restore while the window fills the screen. */
export const Maximized: TStory = {
  args: { isMaximized: true },
};
