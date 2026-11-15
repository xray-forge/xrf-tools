import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverTextRow, IEditorPopoverTextRowProps } from "./EditorPopoverTextRow";

function toValueArgs(value: string): Partial<IEditorPopoverTextRowProps> {
  return { value };
}

function EditorPopoverTextRowStory(args: IEditorPopoverTextRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <EditorPopoverTextRow {...args} onChange={args.onChange ? onChange : undefined} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverTextRow",
  component: EditorPopoverTextRow,
  render: EditorPopoverTextRowStory,
  decorators: [withPopoverSurface],
  argTypes: {
    type: { control: "inline-radio", options: ["text", "number"] },
  },
  args: {
    label: "Section",
    description: "The section the object spawns from",
    value: "stalker",
    onChange: fn(),
  },
} satisfies Meta<typeof EditorPopoverTextRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Text: TStory = {};

export const NumberField: TStory = {
  args: { label: "Exposure", type: "number", value: 1.25 },
};

/** Without a handler the value is only shown, read-only. */
export const ReadOnly: TStory = {
  args: { onChange: undefined },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
