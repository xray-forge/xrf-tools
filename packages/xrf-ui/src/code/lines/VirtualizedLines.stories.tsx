import type { Meta, StoryObj } from "@storybook/react-vite";
import { ESyntaxLanguage, ESyntaxToken } from "@xrf/syntax";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withListingPane } from "#/testing/stories";

import { ECodeLineMark, ICodeLine, ICodeLineSource, toCodeLineSource } from "./code-line";
import { toTextLineSource } from "./text-line-source";
import { IVirtualizedLinesProps, VirtualizedLines } from "./VirtualizedLines";

const ACTOR_LTX: string = `[actor]:identity_immunities
cform                   = skeleton
visual                  = actors\\stalker_neutral\\stalker_neutral_1.ogf
health_restore_v        = 0.0001
max_item_mass           = 50
max_walk_weight         = 60
jump_speed              = 6
crouch_coef             = 0.5
climb_coef              = 0.7
run_coef                = 2.1
sprint_koef             = 2.6

[actor_condition]
satiety_v               = 0.00001
radiation_v             = 0.00004
psy_health_v            = 0.02
bleeding_v              = 0.0004
wound_incarnation_v     = 0.003
min_wound_size          = 0.025
`;

/** Lines a validator flagged, numbered as in the file they were lifted from. */
const FLAGGED_LINES: ReadonlyArray<ICodeLine> = [
  { number: 211, spans: [{ token: ESyntaxToken.SECTION, text: "[wpn_ak74]" }] },
  {
    number: 212,
    spans: [
      { token: ESyntaxToken.KEY, text: "ammo_class" },
      { token: ESyntaxToken.PLAIN, text: " = " },
      { token: ESyntaxToken.STRING, text: "ammo_5.45x39_fmj, ammo_5.45x39_ap" },
    ],
  },
  {
    number: 213,
    mark: ECodeLineMark.WARNING,
    spans: [
      { token: ESyntaxToken.KEY, text: "rpm" },
      { token: ESyntaxToken.PLAIN, text: " = " },
      { token: ESyntaxToken.NUMBER, text: "6000" },
    ],
  },
  {
    number: 214,
    mark: ECodeLineMark.ERROR,
    spans: [
      { token: ESyntaxToken.KEY, text: "visual" },
      { token: ESyntaxToken.PLAIN, text: " = " },
      { token: ESyntaxToken.STRING, text: "dynamics\\weapons\\wpn_ak47\\wpn_ak74.ogf" },
    ],
  },
];

const RESOLVED_LINE_COUNT: number = 551_000;

/** A resolved `system.ltx` as large as Anomaly's, built only as the listing asks for lines. */
const RESOLVED_SYSTEM_LTX: ICodeLineSource = {
  count: RESOLVED_LINE_COUNT,
  layout: {},
  widestNumber: RESOLVED_LINE_COUNT,
  getLine: (index: number): ICodeLine => ({
    number: index + 1,
    spans: [
      { token: ESyntaxToken.KEY, text: `spawn_${String(index).padStart(6, "0")}` },
      { token: ESyntaxToken.PLAIN, text: " = " },
      { token: ESyntaxToken.STRING, text: `l0${(index % 8) + 1}_level, ${index % 360}` },
    ],
  }),
  indexOfLine: (number: number): number => (number >= 1 && number <= RESOLVED_LINE_COUNT ? number - 1 : -1),
};

function toSelectedArgs(line: ICodeLine): Partial<IVirtualizedLinesProps> {
  return { selectedLine: line.number };
}

function VirtualizedLinesStory(args: IVirtualizedLinesProps): ReactElement {
  const onSelectLine = useArgChange(args.onSelectLine, toSelectedArgs);

  return <VirtualizedLines {...args} onSelectLine={onSelectLine} />;
}

const meta = {
  title: "Components/Code/VirtualizedLines",
  component: VirtualizedLines,
  render: VirtualizedLinesStory,
  decorators: [withListingPane],
  argTypes: {
    source: { control: false },
  },
  args: {
    className: "grow surface-content",
    source: toTextLineSource(ACTOR_LTX, ESyntaxLanguage.LTX),
    ariaLabel: "creatures\\actor.ltx",
    selectedLine: 5,
    onSelectLine: fn(),
    onVisibleRangeChange: fn(),
  },
} satisfies Meta<typeof VirtualizedLines>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A coloured config; a click or the arrow keys move the selection. */
export const Config: TStory = {};

/** Error and warning marks beside the numbers of the lines that carry them. */
export const Marked: TStory = {
  args: { source: toCodeLineSource(FLAGGED_LINES), ariaLabel: "weapons\\w_ak74.ltx", selectedLine: 214 },
};

/** Over half a million lines, drawn a window at a time and addressed to one near the end. */
export const Resolved: TStory = {
  args: { source: RESOLVED_SYSTEM_LTX, ariaLabel: "system.ltx", selectedLine: null, scrollToLine: 548_120 },
};
