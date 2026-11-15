import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { MenuItem } from "#/menu";
import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverSelectRow, IEditorPopoverSelectRowProps } from "./EditorPopoverSelectRow";

type TShading = "shaded" | "wireframe" | "normals";

/** An array rather than a fragment: the row's select reads its options as its direct children. */
const SHADING_OPTIONS: ReadonlyArray<ReactElement> = [
  <MenuItem key={"shaded"} value={"shaded"}>
    Shaded
  </MenuItem>,
  <MenuItem key={"wireframe"} value={"wireframe"}>
    Wireframe
  </MenuItem>,
  <MenuItem key={"normals"} value={"normals"}>
    Normals
  </MenuItem>,
];

function toValueArgs(value: TShading): Partial<IEditorPopoverSelectRowProps<TShading>> {
  return { value };
}

function EditorPopoverSelectRowStory(args: IEditorPopoverSelectRowProps<TShading>): ReactElement {
  const onChange = useArgChange(args.onChange, toValueArgs);

  return <EditorPopoverSelectRow<TShading> {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverSelectRow",
  component: EditorPopoverSelectRow<TShading>,
  render: EditorPopoverSelectRowStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Shading",
    description: "How the model's surfaces are drawn",
    value: "shaded",
    children: SHADING_OPTIONS,
    onChange: fn(),
  },
} satisfies Meta<IEditorPopoverSelectRowProps<TShading>>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Default: TStory = {};

export const Disabled: TStory = {
  args: { isDisabled: true },
};
