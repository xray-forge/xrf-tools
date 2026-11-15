import type { Meta, StoryObj } from "@storybook/react-vite";
import { LOGICAL_PATH_SEPARATOR } from "@xrf/path";
import { ReactElement, ReactNode } from "react";
import { fn } from "storybook/test";

import { DescriptionOutlinedIcon } from "#/icons/description-outlined";
import { FolderIcon } from "#/icons/folder";
import { FolderOpenIcon } from "#/icons/folder-open";
import { useArgChange, withPanelSlot } from "#/testing/stories";

import { IPathTreeItem, parsePathTree, toDirectoryItemId, toFileItemId } from "./path-tree";
import { ITreeNode } from "./tree-node";
import { ARCHIVED_CAPTION, TreeRowLabel } from "./TreeRowLabel";
import { IVirtualizedTreeIcons, IVirtualizedTreeProps, VirtualizedTree } from "./VirtualizedTree";

interface IGameDataEntry {
  isArchived: boolean;
}

const PATHS: ReadonlyArray<string> = [
  "config\\system.ltx",
  "config\\creatures\\actor.ltx",
  "config\\creatures\\m_stalker.ltx",
  "config\\weapons\\w_ak74.ltx",
  "config\\weapons\\w_pm.ltx",
  "meshes\\actors\\stalker_bandit\\stalker_bandit_1.ogf",
  "meshes\\actors\\stalker_bandit\\stalker_bandit_2.ogf",
  "scripts\\_g.script",
  "scripts\\xr_logic.script",
  "textures\\act\\act_stalker_bandit.dds",
];

const ITEMS: Array<IPathTreeItem<IGameDataEntry>> = parsePathTree(
  PATHS.map((path: string, index: number) => ({ path, payload: { isArchived: index % 3 === 0 } })),
  LOGICAL_PATH_SEPARATOR
);

const EXPANDED_IDS: ReadonlySet<string> = new Set([
  toDirectoryItemId("config"),
  toDirectoryItemId("config\\creatures"),
]);

const ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DescriptionOutlinedIcon />,
};

function renderLabel(item: ITreeNode<IGameDataEntry>): ReactNode {
  return (
    <TreeRowLabel
      label={item.label}
      caption={item.payload?.isArchived ? ARCHIVED_CAPTION : null}
      captionTitle={"Read from an archive"}
    />
  );
}

function toSelectedArgs(item: ITreeNode<IGameDataEntry>): Partial<IVirtualizedTreeProps<IGameDataEntry>> {
  return { selectedId: item.id };
}

/** Expansion is held in the args rather than by `useTreeState`, whose React state would re-render past `useArgs`. */
function toToggledArgs(
  id: string,
  { expandedIds }: IVirtualizedTreeProps<IGameDataEntry>
): Partial<IVirtualizedTreeProps<IGameDataEntry>> {
  const next: Set<string> = new Set(expandedIds);

  if (!next.delete(id)) {
    next.add(id);
  }

  return { expandedIds: next };
}

function VirtualizedTreeStory(args: IVirtualizedTreeProps<IGameDataEntry>): ReactElement {
  const onSelect = useArgChange(args.onSelect, toSelectedArgs);
  const onToggleExpanded = useArgChange(args.onToggleExpanded, toToggledArgs);

  return <VirtualizedTree<IGameDataEntry> {...args} onSelect={onSelect} onToggleExpanded={onToggleExpanded} />;
}

const meta = {
  title: "Components/Tree/VirtualizedTree",
  component: VirtualizedTree<IGameDataEntry>,
  render: VirtualizedTreeStory,
  decorators: [withPanelSlot],
  argTypes: {
    items: { control: false },
    expandedIds: { control: false },
  },
  args: {
    items: ITEMS,
    expandedIds: EXPANDED_IDS,
    selectedId: null,
    activeId: toFileItemId("config\\creatures\\actor.ltx"),
    ariaLabel: "Game data",
    icons: ICONS,
    renderLabel,
    onToggleExpanded: fn(),
    onSelect: fn(),
    onActivate: fn(),
  },
} satisfies Meta<IVirtualizedTreeProps<IGameDataEntry>>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Folders, files and the archive caption; the open file is marked apart from the keyboard cursor. */
export const PathTree: TStory = {};

/** A tree whose rows are all one kind of thing draws the chevron alone. */
export const Plain: TStory = {
  args: { icons: undefined, renderLabel: undefined },
};
