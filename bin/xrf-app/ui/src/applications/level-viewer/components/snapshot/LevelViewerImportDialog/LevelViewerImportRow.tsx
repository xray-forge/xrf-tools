import { ABSENT_VALUE } from "@xrf/format";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Checkbox, Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme/class-name";
import { ChangeEvent, ReactElement, useCallback } from "react";

import {
  ELevelViewerSnapshotStatus,
  ILevelViewerSnapshotEntry,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";

/**
 * @param key - The value's key, which says how it reads: the weather's clock as a time of day.
 * @param value - The value, undefined for none.
 * @returns The value as the dialog shows it.
 */
function formatLevelViewerImportValue(key: string, value: unknown): string {
  if (value === undefined) {
    return ABSENT_VALUE;
  }

  if (key === "weather.time" && typeof value === "number") {
    return formatLevelWeatherTime(value);
  }

  if (typeof value === "boolean") {
    return value ? "on" : "off";
  }

  if (typeof value === "number") {
    return String(Math.round(value * 1000) / 1000);
  }

  if (Array.isArray(value)) {
    return value.map((item: unknown) => formatLevelViewerImportValue(key, item)).join(", ");
  }

  return value === null ? "none" : String(value);
}

export interface ILevelViewerImportRowProps extends BaseComponentProps {
  entry: ILevelViewerSnapshotEntry;
  /** Whether it is chosen to apply; a row without `onToggle` offers no choice. */
  isChosen?: boolean;
  onToggle?: (key: string, isChosen: boolean) => void;
}

/**
 * One value of a snapshot: its key, what the view has, and what the file sets, or why the view does not take it.
 */
export function LevelViewerImportRow({
  "data-testid": dataTestId = "level-viewer-import-row",
  id,
  className,
  entry,
  isChosen,
  onToggle,
}: ILevelViewerImportRowProps): ReactElement {
  const isSkipped: boolean = entry.status === ELevelViewerSnapshotStatus.SKIP;
  const isBounded: boolean =
    entry.status === ELevelViewerSnapshotStatus.CHANGE && JSON.stringify(entry.applied) !== JSON.stringify(entry.value);

  const onChange = useCallback(
    (_: ChangeEvent<HTMLInputElement>, isChecked: boolean): void => onToggle?.(entry.key, isChecked),
    [onToggle, entry.key]
  );

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn(
        "grid grid-cols-[1.75rem_minmax(0,1fr)_minmax(0,8rem)_1rem_minmax(0,12rem)] items-center gap-x-2",
        "border-t border-divider py-0.5",
        className
      )}
    >
      {onToggle ? (
        <Checkbox
          className={"p-0.5"}
          size={"small"}
          checked={Boolean(isChosen)}
          slotProps={{ input: { "aria-label": `Apply ${entry.key}` } }}
          onChange={onChange}
        />
      ) : (
        <span />
      )}

      <Typography className={"monospace truncate"} variant={"body2"} title={entry.key}>
        {entry.key.slice(entry.group.length + 1) || entry.key}
      </Typography>

      <Typography className={"truncate text-right text-text-secondary"} variant={"body2"}>
        {formatLevelViewerImportValue(entry.key, entry.current)}
      </Typography>

      <Typography className={"text-center text-text-secondary"} variant={"body2"}>
        →
      </Typography>

      <div className={"flex min-w-0 flex-col"}>
        <Typography className={cn("truncate", isSkipped && "text-text-secondary line-through")} variant={"body2"}>
          {formatLevelViewerImportValue(entry.key, entry.applied ?? entry.value)}
          {isBounded ? (
            <Typography className={"ml-2 text-warning"} component={"span"} variant={"caption"}>
              {`file ${formatLevelViewerImportValue(entry.key, entry.value)}`}
            </Typography>
          ) : null}
        </Typography>

        {entry.reason ? (
          <Typography className={"truncate text-text-secondary"} variant={"caption"} title={entry.reason}>
            {entry.reason}
          </Typography>
        ) : null}
      </div>
    </div>
  );
}
