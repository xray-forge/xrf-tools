import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withFrameStrip } from "#/testing/stories";

import { CloseGlyph } from "./CloseGlyph";
import { MaximizeGlyph } from "./MaximizeGlyph";
import { MenuGlyph } from "./MenuGlyph";
import { MinimizeGlyph } from "./MinimizeGlyph";
import { RestoreGlyph } from "./RestoreGlyph";
import { WindowControlButton } from "./WindowControlButton";

const meta = {
  title: "Components/Shell/WindowControlButton",
  component: WindowControlButton,
  decorators: [withFrameStrip],
  args: {
    label: "Minimize",
    children: <MinimizeGlyph />,
    onClick: fn(),
  },
} satisfies Meta<typeof WindowControlButton>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Minimize: TStory = {};

export const Maximize: TStory = {
  args: { label: "Maximize", children: <MaximizeGlyph /> },
};

export const Restore: TStory = {
  args: { label: "Restore", children: <RestoreGlyph /> },
};

export const Menu: TStory = {
  args: { label: "Application menu", children: <MenuGlyph /> },
};

/** Tinted red on hover in both schemes, as Windows does only the close button. */
export const Close: TStory = {
  args: { label: "Close", isDanger: true, children: <CloseGlyph /> },
};

/** The caption's three, as the window draws them at its right edge. */
export const Caption: TStory = {
  decorators: [
    (Story) => (
      <>
        <WindowControlButton label={"Minimize"} onClick={fn()}>
          <MinimizeGlyph />
        </WindowControlButton>
        <WindowControlButton label={"Maximize"} onClick={fn()}>
          <MaximizeGlyph />
        </WindowControlButton>
        <Story />
      </>
    ),
  ],
  args: { label: "Close", isDanger: true, children: <CloseGlyph /> },
};
