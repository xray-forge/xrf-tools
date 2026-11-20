import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { withPanelSlot } from "#/testing/stories";

import { EditorProblemsList, IEditorProblem } from "./EditorProblemsList";

const FINDINGS: Array<IEditorProblem> = [
  {
    rule: "ltx.unknown-section",
    subject: "configs\\gameplay\\character_desc_escape.xml",
    message: "Names a section no config declares: esc_bandit_trader.",
    location: { path: "configs\\gameplay\\character_desc_escape.xml", line: 42 },
  },
  {
    rule: "ltx.duplicate-field",
    subject: "configs\\weapons\\w_ak74.ltx",
    message: "Writes hit_power twice in [wpn_ak74]; the second wins.",
    location: { path: "configs\\weapons\\w_ak74.ltx", line: 118 },
  },
  { rule: "ltx.read-failed", subject: null, message: "Could not read configs\\misc\\broken.ltx." },
];

const meta = {
  title: "Components/Editor/EditorProblemsList",
  component: EditorProblemsList,
  decorators: [withPanelSlot],
  argTypes: {
    findings: { control: false },
  },
  args: {
    findings: FINDINGS,
    rulePrefix: "ltx.",
    emptyDescription: "Every config in this project read cleanly.",
    onSelect: fn(),
  },
} satisfies Meta<typeof EditorProblemsList>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Rows naming a place open it; the one without a location stays plain. */
export const Findings: TStory = {};

export const Empty: TStory = {
  args: { findings: [] },
};

/** Findings still being read, said above whatever is already in. */
export const Reading: TStory = {
  args: { status: "Reading the rest of the project…" },
};
