import { Typography } from "@mui/material";
import { ReactElement, ReactNode, useMemo } from "react";

import { MONOSPACE } from "@/core/theme/tokens";
import { EStatMeasure } from "@/core/ui/stats/stat-measure";
import { StatBar } from "@/core/ui/stats/StatBar";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

export interface IStatBreakdownRow {
  /** Identifies the row, and is what a click reports.  */
  id: string;
  label: string;
  /** Said beside the label where the key alone does not carry it: a ratio, a mount kind, an unknown spelling. */
  note?: ReactNode;
  files: number;
  sizeReal: number;
}

export interface IStatBreakdownTableProps extends BaseComponentProps {
  rows: Array<IStatBreakdownRow>;
  /** Which measurement orders the rows, draws the bars, and takes the emphasis. */
  measure: EStatMeasure;
  /** Keeps the rows in the order given instead of ordering them by `measure`. */
  isPreordered?: boolean;
  /** The row a listing below is scoped to, drawn as chosen. Only a selectable table has one. */
  selectedId?: Nullable<string>;
  /** A row was chosen, by its id. Given the row already chosen, so a second click can clear it. */
  onSelect?: (id: string) => void;
}

/**
 * A breakdown as rows of label, proportion, and both measurements.
 */
export function StatBreakdownTable({
  "data-testid": dataTestId = "stat-breakdown-table",
  id,
  className,
  rows,
  measure,
  isPreordered = false,
  selectedId = null,
  onSelect,
}: IStatBreakdownTableProps): ReactElement {
  const totals: { files: number; sizeReal: number } = useMemo(
    () =>
      rows.reduce(
        (total: { files: number; sizeReal: number }, row: IStatBreakdownRow) => ({
          files: total.files + row.files,
          sizeReal: total.sizeReal + row.sizeReal,
        }),
        { files: 0, sizeReal: 0 }
      ),
    [rows]
  );

  const isBytes: boolean = measure === EStatMeasure.BYTES;

  const ordered: Array<IStatBreakdownRow> = useMemo(() => {
    if (isPreordered) {
      return rows;
    }

    // Copied before sorting, because the rows belong to whatever derived them. The label breaks ties so two readings
    // of one report put equal rows in the same place.
    return [...rows].sort((first: IStatBreakdownRow, second: IStatBreakdownRow) => {
      const difference: number = isBytes ? second.sizeReal - first.sizeReal : second.files - first.files;

      return difference === 0 ? first.label.localeCompare(second.label) : difference;
    });
  }, [rows, isBytes, isPreordered]);

  return (
    <div data-testid={dataTestId} id={id} className={cn("grid gap-y-1", className)}>
      {ordered.map((row: IStatBreakdownRow) => (
        <div
          key={row.id}
          data-testid={"stat-breakdown-row"}
          aria-pressed={onSelect ? row.id === selectedId : undefined}
          className={cn(
            "grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-x-3 rounded-surface p-1 aria-pressed:bg-action-selected aria-pressed:hover:bg-action-selected",
            onSelect && "cursor-pointer hover:bg-action-hover"
          )}
          role={onSelect ? "button" : undefined}
          onClick={onSelect ? () => onSelect(row.id) : undefined}
        >
          <div className={"min-w-0"}>
            <div className={"flex min-w-0 items-baseline gap-1.5"}>
              <Typography
                variant={"body2"}
                sx={{ ...MONOSPACE, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
              >
                {row.label}
              </Typography>

              {row.note}
            </div>

            <StatBar
              value={isBytes ? row.sizeReal : row.files}
              total={isBytes ? totals.sizeReal : totals.files}
              label={`${row.label}: ${isBytes ? formatBytes(row.sizeReal) : `${row.files} files`}`}
            />
          </div>

          <Typography
            className={"w-18"}
            variant={"caption"}
            sx={{ textAlign: "right", color: isBytes ? "text.secondary" : "text.primary" }}
          >
            {row.files.toLocaleString()}
          </Typography>

          <Typography
            className={"w-22.5"}
            variant={"caption"}
            sx={{ textAlign: "right", color: isBytes ? "text.primary" : "text.secondary" }}
          >
            {formatBytes(row.sizeReal)}
          </Typography>
        </div>
      ))}
    </div>
  );
}
