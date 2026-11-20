import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EditorPopoverCheckboxRow } from "#/editor/popover-row";
import { WbSunnyIcon } from "#/icons/wb-sunny";
import { withFrameStrip } from "#/testing/stories";

import { EditorPopoverGroup } from "./EditorPopoverGroup";
import { EditorPopoverGroupSection } from "./EditorPopoverGroupSection";

const LIGHTING_SECTIONS: ReactElement = (
  <>
    <EditorPopoverGroupSection label={"Sun"} isOn={true} onToggle={fn()}>
      <EditorPopoverCheckboxRow label={"Shadows"} isChecked={true} onChange={fn()} />
    </EditorPopoverGroupSection>
    <EditorPopoverGroupSection label={"Lightmaps"} isOn={false} onToggle={fn()} />
  </>
);

const meta = {
  title: "Components/Editor/EditorPopoverGroup",
  component: EditorPopoverGroup,
  decorators: [withFrameStrip],
  args: {
    label: "Lighting",
    description: "Sun on, lightmaps off",
    icon: <WbSunnyIcon />,
    isActive: true,
    children: LIGHTING_SECTIONS,
  },
} satisfies Meta<typeof EditorPopoverGroup>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** One trigger over several toggles, painted on while any of them is; click it to open them. */
export const Active: TStory = {};

export const Inactive: TStory = {
  args: { isActive: false, description: "Everything off" },
};

export const WithReset: TStory = {
  args: { onReset: fn(), resetLabel: "Reset the lighting" },
};
