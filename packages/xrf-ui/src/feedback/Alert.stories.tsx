import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withReadingColumn } from "#/testing/stories";

import { Alert } from ".";

const meta = {
  title: "Components/Feedback/Alert",
  component: Alert,
  decorators: [withReadingColumn],
  argTypes: {
    severity: { control: "inline-radio", options: ["success", "info", "warning", "error"] },
    variant: { control: "inline-radio", options: ["standard", "outlined", "filled"] },
  },
  args: {
    severity: "error",
    children: "The archive could not be read: the header names 1,204 entries and the file ends after 1,150.",
  },
} satisfies Meta<typeof Alert>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** An error that stops the work; a note beside rows that only informs is an `EditorPopoverNote`. */
export const Failure: TStory = {};

export const Warning: TStory = {
  args: { severity: "warning", children: "3 entries are written twice; the later copy wins." },
};

export const Success: TStory = {
  args: { severity: "success", children: "Unpacked 38,113 files." },
};

export const Dismissible: TStory = {
  args: { onClose: fn() },
};
