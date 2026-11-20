import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverNumberRow, IEditorPopoverNumberRowProps } from "./EditorPopoverNumberRow";

function toValueArgs(value: number): Partial<IEditorPopoverNumberRowProps> {
  return { value };
}

function EditorPopoverNumberRowStory(args: IEditorPopoverNumberRowProps): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <EditorPopoverNumberRow {...args} onChange={args.onChange ? onChange : undefined} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverNumberRow",
  component: EditorPopoverNumberRow,
  render: EditorPopoverNumberRowStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Bump height",
    description: "How deep the parallax reads",
    value: 0.05,
    onChange: fn(),
  },
} satisfies Meta<typeof EditorPopoverNumberRow>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

/** A value there is no number for, such as a non-finite one a file held, shows empty. */
export const NoNumber: TStory = {
  args: { value: null },
};

/** Without a handler the value is only shown, read-only. */
export const ReadOnly: TStory = {
  args: { onChange: undefined },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
