import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverCheckboxRow, IEditorPopoverCheckboxRowProps } from "./EditorPopoverCheckboxRow";

function toCheckedArgs(isChecked: boolean): Partial<IEditorPopoverCheckboxRowProps> {
  return { isChecked };
}

function EditorPopoverCheckboxRowStory(args: IEditorPopoverCheckboxRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toCheckedArgs);

  return <EditorPopoverCheckboxRow {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverCheckboxRow",
  component: EditorPopoverCheckboxRow,
  render: EditorPopoverCheckboxRowStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Cull back faces",
    description: "Skip the faces turned away from the camera",
    isChecked: true,
    onChange: fn(),
  },
} satisfies Meta<typeof EditorPopoverCheckboxRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A setting in the popover's label column, its box beside it where a value would be. */
export const Checked: TStory = {};

export const Clear: TStory = {
  args: { isChecked: false },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
