import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";

import { LayersIcon } from "#/icons/layers";
import { VisibilityIcon } from "#/icons/visibility";
import { withPopoverSurface } from "#/testing/stories";

import {
  EditorPopoverCheckboxTable,
  IEditorPopoverCheckboxColumn,
  IEditorPopoverCheckboxTableRow,
} from "./EditorPopoverCheckboxTable";

const COLUMNS: ReadonlyArray<IEditorPopoverCheckboxColumn> = [
  { id: "visible", label: "Visible", icon: <VisibilityIcon /> },
  { id: "labelled", label: "Labelled", icon: <LayersIcon /> },
];

const ROWS: ReadonlyArray<IEditorPopoverCheckboxTableRow> = [
  {
    id: "stalkers",
    label: "Stalkers",
    description: "Every human spawn",
    cells: [
      { isChecked: true, onChange: fn() },
      { isChecked: false, onChange: fn() },
    ],
  },
  {
    id: "monsters",
    label: "Monsters",
    cells: [
      { isChecked: true, onChange: fn() },
      { isChecked: true, isDisabled: true, reason: "Monsters carry no name to label", onChange: fn() },
    ],
  },
  { id: "patrols", label: "Patrol paths", cells: [{ isChecked: false, onChange: fn() }, null] },
];

const meta = {
  title: "Components/Editor/EditorPopoverCheckboxTable",
  component: EditorPopoverCheckboxTable,
  decorators: [withPopoverSurface],
  args: {
    columns: COLUMNS,
    rows: ROWS,
  },
} satisfies Meta<typeof EditorPopoverCheckboxTable>;

export default meta;

type TStory = StoryObj<typeof meta>;

/** Rows of switches under icon columns; a disabled box says why on hover, a row without a cell leaves it blank. */
export const Default: TStory = {};
