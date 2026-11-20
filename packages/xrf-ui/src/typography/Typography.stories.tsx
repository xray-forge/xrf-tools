import type { Meta, StoryObj } from "@storybook/react-vite";

import { withReadingColumn } from "#/testing/stories";

import { Typography } from ".";

const meta = {
  title: "Components/Typography/Typography",
  component: Typography,
  decorators: [withReadingColumn],
  argTypes: {
    variant: {
      control: "select",
      options: ["h5", "h6", "subtitle1", "subtitle2", "body1", "body2", "button", "caption", "overline"],
    },
  },
  args: {
    variant: "body2",
    children: "Writes every entry of the selected archives under one directory.",
  },
} satisfies Meta<typeof Typography>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Body: TStory = {};

export const Heading: TStory = {
  args: { variant: "h6", children: "Unpack archives" },
};

export const Caption: TStory = {
  args: { variant: "caption", className: "text-text-secondary" },
};

/** `noWrap` ends in an ellipsis on every variant: the theme makes the inline ones block so it can. */
export const NoWrap: TStory = {
  args: { variant: "caption", noWrap: true, children: "meshes\\actors\\stalker_bandit\\".repeat(8) },
};

/** Identifiers, engine paths and entry names take the theme's `monospace` class. */
export const Monospace: TStory = {
  args: { className: "monospace", children: "config\\creatures\\actor.ltx" },
};
