import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { SaveIcon } from "#/icons/save";
import { UnarchiveIcon } from "#/icons/unarchive";
import { withFrameStrip } from "#/testing/stories";

import { EditorTextAction } from "./EditorTextAction";

const meta = {
  title: "Components/Editor/EditorTextAction",
  component: EditorTextAction,
  decorators: [withFrameStrip],
  argTypes: {
    variant: { control: "inline-radio", options: ["outlined", "contained", "text"] },
  },
  args: {
    label: "Unpack",
    description: "Unpack the selected archives under the output directory",
    icon: <UnarchiveIcon />,
    onClick: fn(),
  },
} satisfies Meta<typeof EditorTextAction>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Outlined: TStory = {};

export const Contained: TStory = {
  args: { label: "Save", description: "Write the changes to gamedata", icon: <SaveIcon />, variant: "contained" },
};

export const Text: TStory = {
  args: { variant: "text", icon: undefined },
};

/** The tooltip still answers while disabled, saying why. */
export const Disabled: TStory = {
  args: { isDisabled: true, description: "Select an archive to unpack" },
};
