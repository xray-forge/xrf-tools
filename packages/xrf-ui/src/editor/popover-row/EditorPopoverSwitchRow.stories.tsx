import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverSwitchRow, IEditorPopoverSwitchRowProps } from "./EditorPopoverSwitchRow";

function toCheckedArgs(isChecked: boolean): Partial<IEditorPopoverSwitchRowProps> {
  return { isChecked };
}

function EditorPopoverSwitchRowStory(args: IEditorPopoverSwitchRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toCheckedArgs);

  return <EditorPopoverSwitchRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverSwitchRow",
  component: EditorPopoverSwitchRow,
  render: EditorPopoverSwitchRowStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Show grid",
    description: "Draw the ground grid under the model",
    isChecked: true,
    onChange: fn(),
  },
} satisfies Meta<typeof EditorPopoverSwitchRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** One of a show menu's switches: the box before its name, the name taking the row. */
export const Checked: TStory = {};

/** A switch that only matters while the one before it is on, set in under it. */
export const Nested: TStory = {
  args: { label: "Minor lines", isNested: true },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
