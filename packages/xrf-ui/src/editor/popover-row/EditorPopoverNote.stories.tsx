import type { Meta, StoryObj } from "@storybook/react-vite";

import { withPopoverSurface } from "#/testing/stories";

import { EditorPopoverNote } from "./EditorPopoverNote";

const meta = {
  title: "Components/Editor/EditorPopoverNote",
  component: EditorPopoverNote,
  decorators: [withPopoverSurface],
  argTypes: {
    tone: { control: "inline-radio", options: ["info", "warning"] },
  },
  args: {
    children: "Lightmaps draw only once the level has been compiled with them.",
  },
} satisfies Meta<typeof EditorPopoverNote>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Info: TStory = {};

/** What the rows near it hold will not do what it seems to. */
export const Warning: TStory = {
  args: { tone: "warning", children: "The engine caps this at 4; higher values draw the same." },
};
