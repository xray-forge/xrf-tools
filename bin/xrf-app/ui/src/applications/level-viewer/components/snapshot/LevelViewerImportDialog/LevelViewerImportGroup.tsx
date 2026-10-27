import { Checkbox, Typography } from "@mui/material";
import { formatCount } from "@xrf/format";
import { ChangeEvent, ReactElement, useCallback, useMemo } from "react";

import { LevelViewerImportRow } from "@/applications/level-viewer/components/snapshot/LevelViewerImportDialog/LevelViewerImportRow";
import { ILevelViewerSnapshotEntry } from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** The parts of a snapshot that are not renderer settings, by their key, as the dialog heads them. */
const GROUP_LABELS: Readonly<Record<string, string>> = {
  camera: "Camera",
  look: "Look",
  options: "Toolbar",
  output: "Output",
  settings: "Renderer",
  weather: "Weather",
};

/**
 * @param group - A part of the view as entries name it: `camera`, `settings.ambientOcclusion`.
 * @returns Its heading: a fixed name, or a renderer group's key in words.
 */
function toLevelViewerImportGroupLabel(group: string): string {
  const words: string = group
    .replace(/^settings\./, "")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .toLowerCase();

  return GROUP_LABELS[group] ?? `${words.charAt(0).toUpperCase()}${words.slice(1)}`;
}

export interface ILevelViewerImportGroupProps extends BaseComponentProps {
  group: string;
  /** The part's entries that apply. */
  entries: ReadonlyArray<ILevelViewerSnapshotEntry>;
  /** The keys chosen to apply, of every part. */
  chosen: ReadonlySet<string>;
  onToggle: (keys: ReadonlyArray<string>, isChosen: boolean) => void;
}

/**
 * A part of the view an import changes: a heading choosing all of it, then each value it changes.
 */
export function LevelViewerImportGroup({
  "data-testid": dataTestId = "level-viewer-import-group",
  id,
  className,
  group,
  entries,
  chosen,
  onToggle,
}: ILevelViewerImportGroupProps): ReactElement {
  const label: string = toLevelViewerImportGroupLabel(group);
  const keys: Array<string> = useMemo(() => entries.map((it: ILevelViewerSnapshotEntry) => it.key), [entries]);
  const count: number = keys.filter((key: string) => chosen.has(key)).length;

  const onToggleAll = useCallback(
    (_: ChangeEvent<HTMLInputElement>, isChecked: boolean): void => onToggle(keys, isChecked),
    [onToggle, keys]
  );

  const onToggleEntry = useCallback((key: string, isChosen: boolean): void => onToggle([key], isChosen), [onToggle]);

  return (
    <section data-testid={dataTestId} id={id} className={className} aria-label={label}>
      <div className={"grid grid-cols-[1.75rem_minmax(0,1fr)_auto] items-center gap-x-2 pt-2 pb-0.5"}>
        <Checkbox
          className={"p-0.5"}
          size={"small"}
          checked={count === keys.length}
          indeterminate={count > 0 && count < keys.length}
          slotProps={{ input: { "aria-label": `Apply ${label}` } }}
          onChange={onToggleAll}
        />

        <Typography variant={"subtitle2"}>{label}</Typography>

        <Typography className={"text-text-secondary"} variant={"caption"}>
          {`${formatCount(count)} of ${formatCount(keys.length)}`}
        </Typography>
      </div>

      {entries.map((entry: ILevelViewerSnapshotEntry) => (
        <LevelViewerImportRow key={entry.key} entry={entry} isChosen={chosen.has(entry.key)} onToggle={onToggleEntry} />
      ))}
    </section>
  );
}
