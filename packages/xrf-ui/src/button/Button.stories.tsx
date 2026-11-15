import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { SaveIcon } from "#/icons/save";

import { Button } from ".";

const meta = {
  title: "Components/Button/Button",
  component: Button,
  argTypes: {
    variant: { control: "inline-radio", options: ["text", "outlined", "contained"] },
    color: { control: "select", options: ["primary", "secondary", "success", "warning", "error", "inherit"] },
    size: { control: "inline-radio", options: ["small", "medium", "large"] },
  },
  args: {
    children: "Unpack",
    variant: "contained",
    color: "primary",
    size: "small",
    onClick: fn(),
  },
} satisfies Meta<typeof Button>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Contained: TStory = {};

export const Outlined: TStory = {
  args: { variant: "outlined" },
};

export const Text: TStory = {
  args: { variant: "text", children: "Cancel" },
};

/** What overwrites or deletes, as `ConfirmDialog` paints its confirming button. */
export const Destructive: TStory = {
  args: { color: "error", children: "Overwrite" },
};

export const WithIcon: TStory = {
  args: { startIcon: <SaveIcon />, children: "Save" },
};

export const Disabled: TStory = {
  args: { disabled: true },
};
