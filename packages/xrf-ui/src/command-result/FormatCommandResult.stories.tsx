import type { Meta, StoryObj } from "@storybook/react-vite";

import { withListingPane } from "#/testing/stories";

import { FormatCommandResult, IFormatCommandNouns, IFormatCommandOutcome } from "./FormatCommandResult";

const LTX_NOUNS: IFormatCommandNouns = { item: "file", column: "File", everything: "files" };

const UNFORMATTED: Array<string> = [
  "config\\system.ltx",
  "config\\creatures\\actor.ltx",
  "config\\creatures\\m_stalker.ltx",
  "config\\weapons\\w_ak74.ltx",
  "config\\weapons\\w_pm.ltx",
  "config\\misc\\artefacts.ltx",
  "config\\gameplay\\character_desc_escape.xml",
];

const CHECKED: IFormatCommandOutcome = {
  totalFiles: 1_460,
  validFiles: 1_460 - UNFORMATTED.length,
  invalidFiles: UNFORMATTED.length,
  toFormat: UNFORMATTED,
  duration: 1_840,
  startupDuration: 210,
};

const CLEAN: IFormatCommandOutcome = {
  ...CHECKED,
  validFiles: CHECKED.totalFiles,
  invalidFiles: 0,
  toFormat: [],
};

const meta = {
  title: "Components/CommandResult/FormatCommandResult",
  component: FormatCommandResult,
  decorators: [withListingPane],
  argTypes: {
    result: { control: false },
  },
  args: {
    isCheck: true,
    nouns: LTX_NOUNS,
    result: CHECKED,
  },
} satisfies Meta<typeof FormatCommandResult>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** In check mode a file needing formatting is a failure, listed by path. */
export const Check: TStory = {};

/** The same files rewritten: work done, in the warning's tone rather than the error's. */
export const Write: TStory = {
  args: { isCheck: false },
};

export const Clean: TStory = {
  args: { result: CLEAN },
};
