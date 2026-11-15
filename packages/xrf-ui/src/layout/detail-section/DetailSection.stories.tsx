import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EStatMeasure, IStatBreakdownRow, StatBreakdownTable, StatMeasureToggle } from "#/stats";
import { withReadingColumn } from "#/testing/stories";

import { DetailSection } from "./DetailSection";

const ARCHIVES: Array<IStatBreakdownRow> = [
  { id: "resources.db0", label: "resources.db0", files: 9_870, sizeReal: 1_920_000_000 },
  { id: "resources.db1", label: "resources.db1", files: 6_412, sizeReal: 1_480_000_000 },
  { id: "sounds.db0", label: "sounds.db0", files: 18_412, sizeReal: 2_310_000_000 },
  { id: "configs.db", label: "configs.db", files: 3_419, sizeReal: 96_000_000 },
];

const BREAKDOWN: ReactElement = <StatBreakdownTable rows={ARCHIVES} measure={EStatMeasure.BYTES} />;

const MEASURE_TOGGLE: ReactElement = <StatMeasureToggle measure={EStatMeasure.BYTES} onChange={fn()} />;

const meta = {
  title: "Components/Layout/DetailSection",
  component: DetailSection,
  decorators: [withReadingColumn],
  args: {
    title: "Archives",
    description: "What each volume holds once unpacked; not what it takes on disk.",
    fact: "5.8 GB in 4 archives",
    action: MEASURE_TOGGLE,
    children: BREAKDOWN,
  },
} satisfies Meta<typeof DetailSection>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The title with its fact opposite, one sentence on why beside the section's own control, then its rows. */
export const Default: TStory = {};

/** A heading announcing the rows beneath it, holding none of its own. */
export const Heading: TStory = {
  args: { action: undefined, children: undefined },
};

export const WithoutFact: TStory = {
  args: { fact: null },
};
