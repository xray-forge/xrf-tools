import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EditorPopoverCheckboxRow } from "#/editor/popover-row";
import { useArgChange, withPopoverSurface } from "#/testing/stories";

import { EditorPopoverGroupSection, IEditorPopoverGroupSectionProps } from "./EditorPopoverGroupSection";

const FOG_ROWS: ReactElement = (
  <>
    <EditorPopoverCheckboxRow label={"Height fog"} isChecked={true} onChange={fn()} />
    <EditorPopoverCheckboxRow label={"Sun shafts"} isChecked={false} onChange={fn()} />
  </>
);

/** The handler is told only that a click happened, so the turn reads what it turns over from the args. */
function toToggledArgs({ isOn }: IEditorPopoverGroupSectionProps): Partial<IEditorPopoverGroupSectionProps> {
  return { isOn: !isOn };
}

function EditorPopoverGroupSectionStory(args: IEditorPopoverGroupSectionProps): ReactElement {
  const onToggle = useArgChange(args.onToggle, toToggledArgs);

  return <EditorPopoverGroupSection {...args} onToggle={args.onToggle ? onToggle : undefined} />;
}

const meta = {
  title: "Components/Editor/EditorPopoverGroupSection",
  component: EditorPopoverGroupSection,
  render: EditorPopoverGroupSectionStory,
  decorators: [withPopoverSurface],
  args: {
    label: "Atmosphere",
    description: "Fog and sun shafts over the level",
    isOn: true,
    children: FOG_ROWS,
    onToggle: fn(),
  },
} satisfies Meta<typeof EditorPopoverGroupSection>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A toggle heading its settings, the box before the section's name. */
export const On: TStory = {};

export const WithReset: TStory = {
  args: { onReset: fn(), resetLabel: "Reset the atmosphere" },
};

/** What the mode it stands at draws, said under the heading. */
export const WithNote: TStory = {
  args: { note: "Drawn from the weather the level loads with." },
};

/** Without `onToggle` the section is a titled run of rows. */
export const Titled: TStory = {
  args: { onToggle: undefined },
};

export const Disabled: TStory = {
  args: { isOn: false, isDisabled: true, description: "The level has no weather to draw" },
};
