import type { Meta, StoryObj } from "@storybook/react-vite";

import { withPanelSlot } from "#/testing/stories";

import { EditorPanelEmpty } from "./EditorPanelEmpty";

const meta = {
  title: "Components/Editor/EditorPanelEmpty",
  component: EditorPanelEmpty,
  decorators: [withPanelSlot],
  args: {
    label: "Select an object in the viewport to see its properties.",
  },
} satisfies Meta<typeof EditorPanelEmpty>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};
