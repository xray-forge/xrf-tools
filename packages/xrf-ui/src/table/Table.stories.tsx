import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { withReadingColumn } from "#/testing/stories";

import { Table, TableBody, TableCell, TableHead, TableRow } from ".";

interface IVolumeRow {
  name: string;
  files: string;
  size: string;
}

const VOLUMES: ReadonlyArray<IVolumeRow> = [
  { name: "configs.db", files: "3,112", size: "48 MB" },
  { name: "resources.db", files: "9,870", size: "6.8 GB" },
  { name: "sounds.db", files: "18,412", size: "2.3 GB" },
];

const VOLUME_TABLE: ReactElement = (
  <>
    <TableHead>
      <TableRow>
        <TableCell>Volume</TableCell>
        <TableCell align={"right"}>Files</TableCell>
        <TableCell align={"right"}>Size</TableCell>
      </TableRow>
    </TableHead>
    <TableBody>
      {VOLUMES.map((volume: IVolumeRow) => (
        <TableRow key={volume.name} hover>
          <TableCell className={"monospace"}>{volume.name}</TableCell>
          <TableCell align={"right"}>{volume.files}</TableCell>
          <TableCell align={"right"}>{volume.size}</TableCell>
        </TableRow>
      ))}
    </TableBody>
  </>
);

const meta = {
  title: "Components/Table/Table",
  component: Table,
  decorators: [withReadingColumn],
  argTypes: {
    size: { control: "inline-radio", options: ["small", "medium"] },
  },
  args: {
    size: "small",
    children: VOLUME_TABLE,
  },
} satisfies Meta<typeof Table>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** The plain MUI table, for a short fixed listing; a long or sortable one is a `DataTable`. */
export const Small: TStory = {};

export const Medium: TStory = {
  args: { size: "medium" },
};
