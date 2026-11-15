import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { Button } from "#/button";
import { GridColDef, GridRowId } from "#/data-grid";
import { identifierColumn, pathColumn, textColumn } from "#/table";
import { withListingPane } from "#/testing/stories";

import { CommandResult, ICommandResultStat } from "./CommandResult";
import { CommandResultFindings } from "./CommandResultFindings";

interface IConfigFinding {
  id: number;
  severity: string;
  section: string;
  message: string;
  file: string;
}

const FINDINGS: Array<IConfigFinding> = [
  {
    id: 1,
    severity: "error",
    section: "wpn_ak74",
    message: "visual names a missing mesh",
    file: "config\\weapons\\w_ak74.ltx",
  },
  {
    id: 2,
    severity: "error",
    section: "stalker_bandit_master",
    message: "inherits from undefined section 'stalker_bandit_base'",
    file: "config\\creatures\\m_stalker.ltx",
  },
  {
    id: 3,
    severity: "warning",
    section: "af_medusa",
    message: "cost is set twice",
    file: "config\\misc\\artefacts.ltx",
  },
  {
    id: 4,
    severity: "warning",
    section: "m_bloodsucker_e",
    message: "immunities name an unknown hit type 'telepatic'",
    file: "config\\creatures\\m_bloodsucker.ltx",
  },
];

const COLUMNS: Array<GridColDef<IConfigFinding>> = [
  textColumn("severity", "Severity", 100),
  identifierColumn("section", "Section", 200),
  textColumn("message", "Message", 320),
  pathColumn("file", "File"),
];

const STATS: Array<ICommandResultStat> = [
  { label: "files", value: 1_460 },
  { label: "sections", value: 12_883 },
  { label: "errors", value: 2, tone: "error" },
  { label: "warnings", value: 2, tone: "warning" },
  { label: "elapsed", value: "1.8 s" },
];

const CLEAN_STATS: Array<ICommandResultStat> = [
  { label: "files", value: 1_460 },
  { label: "sections", value: 12_883 },
  { label: "elapsed", value: "1.6 s" },
];

function getRowId(row: IConfigFinding): GridRowId {
  return row.id;
}

function getSearchText(row: IConfigFinding): string {
  return `${row.section} ${row.message} ${row.file}`;
}

const FINDINGS_TABLE: ReactElement = (
  <CommandResultFindings<IConfigFinding>
    rows={FINDINGS}
    columns={COLUMNS}
    getRowId={getRowId}
    getSearchText={getSearchText}
    emptyLabel={"Nothing was found."}
  />
);

const meta = {
  title: "Components/CommandResult/CommandResult",
  component: CommandResult,
  decorators: [withListingPane],
  argTypes: {
    tone: { control: "inline-radio", options: ["success", "warning", "error", "info"] },
    stats: { control: false },
  },
  args: {
    headline: "2 errors and 2 warnings in 1,460 configs",
    tone: "error",
    stats: STATS,
    actions: (
      <Button size={"small"} variant={"outlined"} onClick={fn()}>
        Copy report
      </Button>
    ),
    children: FINDINGS_TABLE,
  },
} satisfies Meta<typeof CommandResult>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The headline in its tone, the stats in theirs, and the findings as a table that sorts and filters. */
export const Findings: TStory = {};

/** A clean run answers in its headline and stats and has nothing below them. */
export const Clean: TStory = {
  args: {
    headline: "Every config is valid",
    tone: "success",
    stats: CLEAN_STATS,
    actions: undefined,
    children: undefined,
  },
};
