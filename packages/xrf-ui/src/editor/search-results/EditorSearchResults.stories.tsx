import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { DescriptionOutlinedIcon } from "#/icons/description-outlined";
import { useArgChange, withPanelSlot } from "#/testing/stories";

import { EditorSearchResults, IEditorSearchResultRow, IEditorSearchResultsProps } from "./EditorSearchResults";

const TEXTURE_MATCHES: Array<IEditorSearchResultRow> = [
  "act_stalker_bandit.dds",
  "act_stalker_bandit_bump.dds",
  "act_stalker_bandit_bump#.dds",
  "act_stalker_bandit_2.dds",
  "act_stalker_bandit_3.dds",
  "act_stalker_bandit_4.dds",
  "act_stalker_bandit_head.dds",
  "act_stalker_bandit_mask.dds",
].map((name: string) => ({
  id: `textures\\act\\${name}`,
  label: name,
  description: "textures\\act",
  icon: <DescriptionOutlinedIcon fontSize={"small"} />,
}));

function toHoveredArgs(activeIndex: number): Partial<IEditorSearchResultsProps<IEditorSearchResultRow>> {
  return { activeIndex };
}

function EditorSearchResultsStory(args: IEditorSearchResultsProps<IEditorSearchResultRow>): ReactElement {
  const onHoverIndex = useArgChange(args.onHoverIndex, toHoveredArgs);

  return <EditorSearchResults<IEditorSearchResultRow> {...args} onHoverIndex={onHoverIndex} />;
}

const meta = {
  title: "Components/Editor/EditorSearchResults",
  component: EditorSearchResults<IEditorSearchResultRow>,
  render: EditorSearchResultsStory,
  decorators: [withPanelSlot],
  argTypes: {
    rows: { control: false },
  },
  args: {
    ariaLabel: "Matching textures",
    emptyLabel: "No texture matches 'stalker_bandit'.",
    rows: TEXTURE_MATCHES,
    total: TEXTURE_MATCHES.length,
    activeIndex: 0,
    onHoverIndex: fn(),
    onSelect: fn(),
  },
} satisfies Meta<IEditorSearchResultsProps<IEditorSearchResultRow>>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The hovered row is the active one, as the arrow keys move it in the panel's field. */
export const Results: TStory = {};

/** More matches than the limit lets through, said above the rows. */
export const Limited: TStory = {
  args: { total: 1_204 },
};

/** The list belongs to an older query than the field: a bar runs above the rows it still shows. */
export const Stale: TStory = {
  args: { isStale: true },
};

/** Stale and empty is "not filtered yet", drawn as the bar alone rather than "nothing matches". */
export const StaleEmpty: TStory = {
  args: { rows: [], total: 0, isStale: true },
};

export const Empty: TStory = {
  args: { rows: [], total: 0 },
};

/** A read is in flight, so no second selection can outrun it. */
export const Disabled: TStory = {
  args: { isDisabled: true },
};
