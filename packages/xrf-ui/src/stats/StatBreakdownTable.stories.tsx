import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { EStatMeasure } from "./stat-measure";
import { IStatBreakdownRow, IStatBreakdownTableProps, StatBreakdownTable } from "./StatBreakdownTable";

const EXTENSIONS: Array<IStatBreakdownRow> = [
  { id: "ogg", label: "ogg", files: 18_412, sizeReal: 2_310_000_000 },
  { id: "dds", label: "dds", files: 9_870, sizeReal: 6_840_000_000 },
  { id: "ogf", label: "ogf", files: 4_203, sizeReal: 1_120_000_000 },
  { id: "geom", label: "geom", files: 34, sizeReal: 3_050_000_000 },
  { id: "ltx", label: "ltx", files: 3_112, sizeReal: 48_000_000 },
  { id: "script", label: "script", files: 1_460, sizeReal: 39_000_000 },
  { id: "xml", label: "xml", files: 1_022, sizeReal: 21_000_000 },
];

/** A second click on the selected row clears it, as the archives' statistics do. */
function toSelectedArgs(id: string, { selectedId }: IStatBreakdownTableProps): Partial<IStatBreakdownTableProps> {
  return { selectedId: selectedId === id ? null : id };
}

function StatBreakdownTableStory(args: IStatBreakdownTableProps): ReactElement {
  const onSelect = useArgChange(args.onSelect, toSelectedArgs);

  return <StatBreakdownTable {...args} onSelect={args.onSelect ? onSelect : undefined} />;
}

const meta = {
  title: "Components/Stats/StatBreakdownTable",
  component: StatBreakdownTable,
  render: StatBreakdownTableStory,
  decorators: [withReadingColumn],
  argTypes: {
    rows: { control: false },
    measure: { control: "inline-radio", options: [EStatMeasure.BYTES, EStatMeasure.COUNT] },
  },
  args: {
    rows: EXTENSIONS,
    measure: EStatMeasure.BYTES,
    selectedId: null,
    onSelect: fn(),
  },
} satisfies Meta<typeof StatBreakdownTable>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Ordered and drawn by bytes; switch `measure` to see the rows reorder and the emphasis move. */
export const Breakdown: TStory = {};

export const ByCount: TStory = {
  args: { measure: EStatMeasure.COUNT },
};

export const ReadOnly: TStory = {
  args: { onSelect: undefined },
};
