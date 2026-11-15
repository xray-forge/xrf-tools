import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withPanelSlot } from "#/testing/stories";

import { EditorFilterInput, IEditorFilterInputProps } from "./EditorFilterInput";

function toQueryArgs(query: string): Partial<IEditorFilterInputProps> {
  return { query };
}

function EditorFilterInputStory(args: IEditorFilterInputProps): ReactElement {
  const onQueryChange = useArgChange(args.onQueryChange, toQueryArgs);

  return <EditorFilterInput {...args} onQueryChange={onQueryChange} />;
}

const meta = {
  title: "Components/Editor/EditorFilterInput",
  component: EditorFilterInput,
  render: EditorFilterInputStory,
  decorators: [withPanelSlot],
  args: {
    ariaLabel: "Filter archive entries",
    query: "",
    placeholder: "Filter by path",
    chord: "Ctrl+F",
    onQueryChange: fn(),
  },
} satisfies Meta<typeof EditorFilterInput>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Empty, with the chord that reaches it standing where the clear button will be. */
export const Empty: TStory = {};

/** A query replaces the chord with the clear button, which hands the caret back to the field. */
export const Filtered: TStory = {
  args: { query: "stalker_bandit" },
};

export const WithoutChord: TStory = {
  args: { chord: undefined },
};
