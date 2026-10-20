import { Checkbox, Typography } from "@mui/material";
import { formatCount } from "@xrf/format";
import { ReactElement } from "react";

import { toLevelViewerImportGroupLabel } from "@/applications/level-viewer/components/snapshot/LevelViewerImportDialog/LevelViewerImportDialog.utils";
import { LevelViewerImportRow } from "@/applications/level-viewer/components/snapshot/LevelViewerImportDialog/LevelViewerImportRow";
import { ILevelViewerSnapshotEntry } from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
  const keys: Array<string> = entries.map((it: ILevelViewerSnapshotEntry) => it.key);
  const count: number = keys.filter((key: string) => chosen.has(key)).length;

  return (
    <section data-testid={dataTestId} id={id} className={className} aria-label={label}>
      <div className={"grid grid-cols-[1.75rem_minmax(0,1fr)_auto] items-center gap-x-2 pt-2 pb-0.5"}>
        <Checkbox
          className={"p-0.5"}
          size={"small"}
          checked={count === keys.length}
          indeterminate={count > 0 && count < keys.length}
          slotProps={{ input: { "aria-label": `Apply ${label}` } }}
          onChange={(_, isChecked: boolean) => onToggle(keys, isChecked)}
        />

        <Typography variant={"subtitle2"}>{label}</Typography>

        <Typography className={"text-text-secondary"} variant={"caption"}>
          {`${formatCount(count)} of ${formatCount(keys.length)}`}
        </Typography>
      </div>

      {entries.map((entry: ILevelViewerSnapshotEntry) => (
        <LevelViewerImportRow
          key={entry.key}
          entry={entry}
          isChosen={chosen.has(entry.key)}
          onToggle={(key: string, isChosen: boolean) => onToggle([key], isChosen)}
        />
      ))}
    </section>
  );
}
