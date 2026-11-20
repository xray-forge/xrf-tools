import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { EditorFilterInput } from "#/editor/filter-input";
import { IEditorSearchResultRow } from "#/editor/search-results";
import { DescriptionOutlinedIcon } from "#/icons/description-outlined";
import { withPanelSlot } from "#/testing/stories";
import { Typography } from "#/typography";

import { EditorSearchMenu, IEditorSearchMenuHeading, IEditorSearchMenuProps } from "./EditorSearchMenu";

interface IStoryTexture {
  path: string;
  name: string;
}

const TEXTURES: Array<IStoryTexture> = [
  "act_stalker_bandit",
  "act_stalker_bandit_bump",
  "act_stalker_bandit_2",
  "act_stalker_freedom",
  "act_stalker_freedom_mask",
  "wpn_ak74",
  "wpn_ak74_bump",
  "wpn_svd",
].map((name: string) => ({ path: `textures\\act\\${name}.dds`, name: `${name}.dds` }));

function toSearchText(texture: IStoryTexture): string {
  return texture.path;
}

function toRow(texture: IStoryTexture): IEditorSearchResultRow {
  return {
    id: texture.path,
    label: texture.name,
    description: "textures\\act",
    icon: <DescriptionOutlinedIcon fontSize={"small"} />,
  };
}

/** A bare stand-in for the panel band the application draws the heading in. */
function renderHeading(heading: IEditorSearchMenuHeading): ReactElement {
  return (
    <div className={"flex flex-col gap-2 border-b border-divider px-3 py-2"}>
      <div className={"flex items-baseline justify-between"}>
        <Typography variant={"subtitle2"}>{heading.title}</Typography>
        <Typography className={"text-text-secondary"} variant={"caption"}>
          {heading.count}
        </Typography>
      </div>
      <EditorFilterInput
        ariaLabel={heading.ariaLabel}
        query={heading.query}
        placeholder={heading.placeholder}
        onClear={heading.onClear}
        onKeyDown={heading.onKeyDown}
        onQueryChange={heading.onQueryChange}
      />
    </div>
  );
}

const meta = {
  title: "Components/Editor/EditorSearchMenu",
  component: EditorSearchMenu<IStoryTexture>,
  decorators: [withPanelSlot],
  argTypes: {
    items: { control: false },
    sections: { control: false },
  },
  args: {
    title: "Textures",
    searchLabel: "Filter textures",
    resultsLabel: "Matching textures",
    items: TEXTURES,
    toSearchText,
    toRow,
    renderHeading,
    onSelect: fn(),
    children: (
      <Typography className={"block px-3 py-2 text-text-secondary"} variant={"body2"}>
        The texture tree, until something is typed.
      </Typography>
    ),
  },
} satisfies Meta<IEditorSearchMenuProps<IStoryTexture>>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Type into the field to rank the textures; the arrow keys and Enter drive the results. */
export const Menu: TStory = {};

/** Filtering and navigation stay while a read is in flight, but no result opens. */
export const ActivationDisabled: TStory = {
  args: { isActivationDisabled: true },
};

/** At most three results reach the list, the rest counted above it. */
export const Limited: TStory = {
  args: { limit: 3 },
};
