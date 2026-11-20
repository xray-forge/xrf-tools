import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { IChoiceFormRowOption } from "#/form/choice-row";
import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverChoiceRow, IEditorPopoverChoiceRowProps } from "./EditorPopoverChoiceRow";

type TQuality = "low" | "medium" | "high";

const QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TQuality>> = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];

const LONG_QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TQuality>> = [
  { value: "low", label: "Hard edges, cheapest" },
  { value: "medium", label: "Filtered edges" },
  { value: "high", label: "Soft contact-hardening edges", isDisabled: true },
];

function toValueArgs(value: TQuality): Partial<IEditorPopoverChoiceRowProps<TQuality>> {
  return { value };
}

function EditorPopoverChoiceRowStory(args: IEditorPopoverChoiceRowProps<TQuality>): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <EditorPopoverChoiceRow<TQuality> {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverChoiceRow",
  component: EditorPopoverChoiceRow<TQuality>,
  render: EditorPopoverChoiceRowStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Shadows",
    description: "How sharp shadows are drawn",
    options: QUALITY_OPTIONS,
    value: "medium",
    onChange: fn(),
  },
} satisfies Meta<IEditorPopoverChoiceRowProps<TQuality>>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

/** Names too long to share the row stand one under another, under the label. */
export const Stacked: TStory = {
  args: { options: LONG_QUALITY_OPTIONS, isStacked: true },
};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
