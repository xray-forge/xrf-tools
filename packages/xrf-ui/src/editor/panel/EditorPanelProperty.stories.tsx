import type { Meta, StoryObj } from "@storybook/react-vite";

import { withPanelSlot, withReadingColumn } from "#/testing/stories";

import { EditorPanelProperty } from "./EditorPanelProperty";

const meta = {
  title: "Components/Editor/EditorPanelProperty",
  component: EditorPanelProperty,
  decorators: [withPanelSlot],
  args: {
    label: "Description",
    value: "A bandit in a leather jacket, the first variant.",
  },
} satisfies Meta<typeof EditorPanelProperty>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The label stacked above a full-width value, as a narrow panel needs. */
export const Stacked: TStory = {};

/** A path folds after its separators rather than mid-name. */
export const Path: TStory = {
  args: { label: "Path", value: "meshes\\actors\\stalker_bandit\\stalker_bandit_1.ogf", isMonospace: true },
};

/** The label in a column of its own beside the value, as a description or a summary lists its facts. */
export const Inline: TStory = {
  decorators: [withReadingColumn],
  args: { label: "Geometry", value: "levels\\l01_escape\\level.geom", isMonospace: true, isInline: true },
};

/** What qualifies the value sits under it. */
export const WithCaption: TStory = {
  decorators: [withReadingColumn],
  args: { label: "Sectors", value: "38", caption: "Portals join the sectors into one level", isInline: true },
};

/** A value addressing something outside the application opens in the system browser. */
export const WithLink: TStory = {
  decorators: [withReadingColumn],
  args: {
    label: "Commit",
    value: "f9925387f",
    href: "https://github.com/xray-forge/xrf-tools/commit/f9925387f",
    isMonospace: true,
    isInline: true,
  },
};
