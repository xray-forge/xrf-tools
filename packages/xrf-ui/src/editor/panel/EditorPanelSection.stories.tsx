import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EditorIconAction } from "#/editor/icon-action";
import { ContentCopyIcon } from "#/icons/content-copy";
import { RefreshIcon } from "#/icons/refresh";
import { withPanelSlot } from "#/testing/stories";

import { EditorPanelProperty } from "./EditorPanelProperty";
import { EditorPanelSection } from "./EditorPanelSection";
import { EditorPanelSectionEmpty } from "./EditorPanelSectionEmpty";

const VISUAL_PROPERTIES: ReactElement = (
  <>
    <EditorPanelProperty label={"Name"} value={"stalker_bandit_1"} isMonospace />
    <EditorPanelProperty label={"Path"} value={"meshes\\actors\\stalker_bandit\\stalker_bandit_1.ogf"} isMonospace />
    <EditorPanelProperty label={"Description"} value={"A bandit in a leather jacket, the first variant."} />
  </>
);

const SOURCE_ACTIONS: ReactElement = (
  <>
    <EditorIconAction
      label={"Copy path"}
      description={"Copy the source path"}
      icon={<ContentCopyIcon />}
      onClick={fn()}
    />
    <EditorIconAction label={"Reload"} description={"Read the file again"} icon={<RefreshIcon />} onClick={fn()} />
  </>
);

const TRANSFORM_SECTION: ReactElement = (
  <EditorPanelSection title={"Transform"}>
    <EditorPanelProperty label={"Position"} value={"12.40, 0.00, -3.75"} isMonospace />
    <EditorPanelProperty label={"Heading"} value={"90.0°"} />
  </EditorPanelSection>
);

const meta = {
  title: "Components/Editor/EditorPanelSection",
  component: EditorPanelSection,
  decorators: [withPanelSlot],
  args: {
    title: "Visual",
    caption: "dynamic, 3 bones",
    children: VISUAL_PROPERTIES,
  },
} satisfies Meta<typeof EditorPanelSection>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Section: TStory = {};

export const WithActions: TStory = {
  args: {
    title: "Source",
    caption: undefined,
    actions: SOURCE_ACTIONS,
    children: <EditorPanelProperty label={"Archive"} value={"gamedata.db0"} isMonospace />,
  },
};

/** After another group, the theme's `panel-section` rules it off from the one before. */
export const Stacked: TStory = {
  decorators: [
    (Story) => (
      <>
        {TRANSFORM_SECTION}
        <Story />
      </>
    ),
  ],
  args: {
    title: "Spawn",
    caption: undefined,
    children: <EditorPanelSectionEmpty label={"This object has no spawn section."} />,
  },
};
