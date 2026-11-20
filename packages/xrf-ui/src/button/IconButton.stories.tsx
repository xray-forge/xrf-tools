import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { RefreshIcon } from "#/icons/refresh";

import { IconButton } from ".";

const meta = {
  title: "Components/Button/IconButton",
  component: IconButton,
  argTypes: {
    color: { control: "select", options: ["default", "primary", "secondary", "error", "inherit"] },
    size: { control: "inline-radio", options: ["small", "medium", "large"] },
  },
  args: {
    "aria-label": "Reload",
    children: <RefreshIcon />,
    onClick: fn(),
  },
} satisfies Meta<typeof IconButton>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Small by default: the theme makes `small` every icon button's size. */
export const Default: TStory = {};

export const Primary: TStory = {
  args: { color: "primary" },
};

export const Medium: TStory = {
  args: { size: "medium" },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
