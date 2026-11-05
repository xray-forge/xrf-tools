import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { Checkbox } from "#/material/checkbox";
import { Tooltip } from "#/material/tooltip";
import { cn } from "#/theme";

/** One column of switches, headed by its icon. */
export interface IEditorPopoverCheckboxColumn {
  id: string;
  /** Names the column in its icon's tooltip, and each of its boxes after the row's name. */
  label: string;
  icon: ReactNode;
}

/** One box of a row, under its column. */
export interface IEditorPopoverCheckboxCell {
  isChecked: boolean;
  isDisabled?: boolean;
  /** Why it cannot be switched, read on hovering it while disabled. */
  reason?: string;
  onChange: (isChecked: boolean) => void;
}

/** One row of switches: its name, and a box under each column, or none where the row has nothing there. */
export interface IEditorPopoverCheckboxTableRow {
  id: string;
  label: string;
  /** What the row stands for, read on hovering its name. */
  description?: string;
  cells: ReadonlyArray<Nullable<IEditorPopoverCheckboxCell>>;
}

export interface IEditorPopoverCheckboxTableProps extends BaseComponentProps {
  columns: ReadonlyArray<IEditorPopoverCheckboxColumn>;
  rows: ReadonlyArray<IEditorPopoverCheckboxTableRow>;
}

/**
 * Several switches for each of many things in one dense table, as an editor's gizmo or visibility menu lists object
 * kinds: a row a thing, a column a switch, each column headed by its icon.
 */
export function EditorPopoverCheckboxTable({
  "data-testid": dataTestId = "editor-popover-checkbox-table",
  id,
  className,
  columns,
  rows,
}: IEditorPopoverCheckboxTableProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col", className)}>
      <div className={"flex h-6 items-center justify-end text-text-secondary"} aria-hidden={true}>
        {columns.map((column: IEditorPopoverCheckboxColumn) => (
          <Tooltip key={column.id} title={column.label}>
            <span className={"flex w-7 justify-center [&>svg]:text-base"}>{column.icon}</span>
          </Tooltip>
        ))}
      </div>

      {rows.map((row: IEditorPopoverCheckboxTableRow) => (
        <div key={row.id} className={"flex min-h-6 items-center rounded-control hover:bg-action-hover"}>
          <Tooltip describeChild title={row.description ?? ""} placement={"top-start"}>
            <span className={"min-w-0 flex-1 truncate text-sm"}>{row.label}</span>
          </Tooltip>

          {columns.map((column: IEditorPopoverCheckboxColumn, index: number) => {
            const cell: Nullable<IEditorPopoverCheckboxCell> = row.cells[index] ?? null;

            return cell ? (
              <Tooltip key={column.id} title={cell.isDisabled ? (cell.reason ?? "") : ""}>
                <span className={"flex w-7 justify-center"}>
                  <Checkbox
                    className={"p-0.5"}
                    size={"small"}
                    checked={cell.isChecked}
                    disabled={cell.isDisabled}
                    slotProps={{ input: { "aria-label": `${row.label} ${column.label.toLowerCase()}` } }}
                    onChange={(_, checked: boolean) => cell.onChange(checked)}
                  />
                </span>
              </Tooltip>
            ) : (
              <span key={column.id} className={"w-7"} />
            );
          })}
        </div>
      ))}
    </div>
  );
}
