import type { Meta, StoryObj } from "@storybook/react-vite";
import { IPoint3 } from "@xrf/math";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { GridColDef, GridRowId } from "#/data-grid";
import { useArgChange, withListingPane } from "#/testing/stories";

import { decimalColumn, flagsColumn, identifierColumn, pathColumn, textColumn, vectorColumn } from "./columns";
import { DataTable, IDataTableProps } from "./DataTable";

interface ISpawnRow {
  id: number;
  section: string;
  name: string;
  visual: string;
  position: IPoint3;
  flags: number;
  health: number;
  level: string;
}

const LEVELS: ReadonlyArray<string> = ["l01_escape", "l02_garbage", "l03_agroprom", "l05_bar"];

const SECTIONS: ReadonlyArray<string> = ["stalker", "m_bloodsucker_e", "m_dog_e", "physic_object", "inventory_box"];

const ROWS: Array<ISpawnRow> = Array.from({ length: 120 }, (_, index: number) => ({
  id: index,
  section: SECTIONS[index % SECTIONS.length],
  name: `${SECTIONS[index % SECTIONS.length]}_${String(index).padStart(4, "0")}`,
  visual: `meshes\\actors\\${SECTIONS[index % SECTIONS.length]}\\variant_${index % 4}.ogf`,
  position: { x: (index * 13.37) % 400, y: (index % 7) * 0.5, z: -((index * 7.13) % 300) },
  flags: (index * 2654435761) >>> 20,
  health: ((index * 37) % 100) / 100,
  level: LEVELS[index % LEVELS.length],
}));

const COLUMNS: Array<GridColDef<ISpawnRow>> = [
  identifierColumn("name", "Name", 220),
  identifierColumn("section", "Section", 160),
  textColumn("level", "Level", 130),
  vectorColumn("position", "Position"),
  decimalColumn("health", "Health", 90),
  flagsColumn("flags", "Flags"),
  pathColumn("visual", "Visual"),
];

function getRowId(row: ISpawnRow): GridRowId {
  return row.id;
}

function getSearchText(row: ISpawnRow): string {
  return `${row.name} ${row.section} ${row.level} ${row.visual}`;
}

function toSelectedRowArgs(row: ISpawnRow): Partial<IDataTableProps<ISpawnRow>> {
  return { selectedRowId: row.id };
}

function DataTableStory(args: IDataTableProps<ISpawnRow>): ReactElement {
  const onRowSelect = useArgChange(args.onRowSelect, toSelectedRowArgs);

  return <DataTable<ISpawnRow> {...args} onRowSelect={onRowSelect} />;
}

const meta = {
  title: "Components/Table/DataTable",
  component: DataTable<ISpawnRow>,
  render: DataTableStory,
  decorators: [withListingPane],
  argTypes: {
    rows: { control: false },
    columns: { control: false },
  },
  args: {
    rows: ROWS,
    columns: COLUMNS,
    getRowId,
    getSearchText,
    emptyLabel: "This level spawns nothing.",
    countNoun: "object",
    hiddenColumns: ["health"],
    selectedRowId: null,
    onRowSelect: fn(),
  },
} satisfies Meta<IDataTableProps<ISpawnRow>>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The compact grid with the kit's column kinds: identifiers, vectors, flags and a path taking the rest of the row. */
export const Spawns: TStory = {};

export const WithoutFilter: TStory = {
  args: { getSearchText: undefined },
};

export const Empty: TStory = {
  args: { rows: [] },
};
