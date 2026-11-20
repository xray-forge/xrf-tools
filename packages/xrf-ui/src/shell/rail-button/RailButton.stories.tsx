import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { ArchiveIcon } from "#/icons/archive";
import { withFrameStrip } from "#/testing/stories";

import { RailButton } from "./RailButton";

const meta = {
  title: "Components/Shell/RailButton",
  component: RailButton,
  decorators: [withFrameStrip],
  argTypes: {
    appearance: { control: "inline-radio", options: ["gradient", "primary", "secondary"] },
    tooltipPlacement: { control: "inline-radio", options: ["right", "left"] },
  },
  args: {
    label: "Archives",
    description: "Unpack, pack and browse the game's archives",
    icon: <ArchiveIcon />,
    onClick: fn(),
  },
} satisfies Meta<typeof RailButton>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The icon filled with the primary-to-secondary gradient. */
export const Rest: TStory = {};

/** Selected drops the gradient for the selected state's colour. */
export const Selected: TStory = {
  args: { isSelected: true },
};

export const Primary: TStory = {
  args: { appearance: "primary" },
};

/** The tooltip says why it is disabled. */
export const Disabled: TStory = {
  args: { isDisabled: true, description: "Open a game folder first" },
};
