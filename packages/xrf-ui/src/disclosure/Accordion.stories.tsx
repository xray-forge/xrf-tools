import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentProps, ReactElement, SyntheticEvent } from "react";
import { fn } from "storybook/test";

import { ExpandMoreIcon } from "#/icons/expand-more";
import { useArgChange, withReadingColumn } from "#/testing/stories";
import { Typography } from "#/typography";

import { Accordion, AccordionDetails, AccordionSummary } from ".";

type TAccordionProps = ComponentProps<typeof Accordion>;

const IMPORT_DETAILS: ReactElement = (
  <>
    <AccordionSummary expandIcon={<ExpandMoreIcon />}>
      <Typography variant={"subtitle2"}>Imports</Typography>
    </AccordionSummary>
    <AccordionDetails>
      <Typography className={"text-text-secondary"} variant={"body2"}>
        weapon_upgrades.ltx and 12 more, resolved against the game data root.
      </Typography>
    </AccordionDetails>
  </>
);

function toExpandedArgs(_event: SyntheticEvent, expanded: boolean): Partial<TAccordionProps> {
  return { expanded };
}

function AccordionStory(args: TAccordionProps): ReactElement {
  const onChange = useArgChange(args.onChange, toExpandedArgs);

  return <Accordion {...args} onChange={onChange} />;
}

const meta = {
  title: "Components/Disclosure/Accordion",
  component: Accordion,
  render: AccordionStory,
  decorators: [withReadingColumn],
  args: {
    children: IMPORT_DETAILS,
    expanded: true,
    onChange: fn(),
  },
} satisfies Meta<typeof Accordion>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** A bordered well, not paper: the theme keeps an inline accordion a recess in whatever hosts it. */
export const Expanded: TStory = {};

export const Collapsed: TStory = {
  args: { expanded: false },
};

export const Disabled: TStory = {
  args: { expanded: false, disabled: true },
};
