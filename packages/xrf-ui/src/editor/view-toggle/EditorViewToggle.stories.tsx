import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { LayersIcon } from "#/icons/layers";
import { useArgChange, withFrameStrip } from "#/testing/stories";

import { EditorViewToggle, IEditorViewToggleProps } from "./EditorViewToggle";

/** The handler is told only that a click happened, so the turn reads what it turns over from the args. */
function toToggledArgs({ isOn }: IEditorViewToggleProps): Partial<IEditorViewToggleProps> {
  return { isOn: !isOn };
}

function EditorViewToggleStory(args: IEditorViewToggleProps): ReactElement {
  const onToggle = useArgChange(args.onToggle, toToggledArgs);

  return <EditorViewToggle {...args} onToggle={onToggle} />;
}

const meta = {
  title: "Components/Editor/EditorViewToggle",
  component: EditorViewToggle,
  render: EditorViewToggleStory,
  decorators: [withFrameStrip],
  args: {
    label: "Alpha channel",
    description: "Draw the texture's alpha channel over a checkerboard",
    icon: <LayersIcon />,
    isOn: true,
    onToggle: fn(),
  },
} satisfies Meta<typeof EditorViewToggle>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Lit while on; the accessible name stays the same either way. */
export const On: TStory = {};

export const Off: TStory = {
  args: { isOn: false },
};

/** The tooltip says why the toggle is not offered in place of what it does. */
export const Unavailable: TStory = {
  args: { isOn: false, isDisabled: true, unavailableTitle: "ston_beton05.dds has no alpha channel" },
};
