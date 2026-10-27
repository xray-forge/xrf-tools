import { formatCount, formatInstant } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { DialogHeader } from "@xrf/ui/dialog/header";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FileOpenOutlinedIcon as FileOpenIcon } from "@xrf/ui/icons/file-open-outlined";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Alert } from "@xrf/ui/material/alert";
import { Button } from "@xrf/ui/material/button";
import { Dialog, DialogActions, DialogContent } from "@xrf/ui/material/dialog";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback, useId, useMemo, useState } from "react";

import { LevelViewerImportGroup } from "@/applications/level-viewer/components/snapshot/LevelViewerImportDialog/LevelViewerImportGroup";
import { LevelViewerImportRow } from "@/applications/level-viewer/components/snapshot/LevelViewerImportDialog/LevelViewerImportRow";
import {
  ELevelViewerSnapshotStatus,
  ILevelViewerSnapshot,
  ILevelViewerSnapshotApplied,
  ILevelViewerSnapshotEntry,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";

/** A snapshot's values by what importing them does, and its changes by part. */
interface ILevelViewerImportSort {
  byStatus: Record<ELevelViewerSnapshotStatus, Array<ILevelViewerSnapshotEntry>>;
  groups: Map<string, Array<ILevelViewerSnapshotEntry>>;
}

export interface ILevelViewerImportDialogProps extends BaseComponentProps {
  /** The file's name, which the summary starts with. */
  fileName: string;
  snapshot: ILevelViewerSnapshot;
  /** Every value of the file, as classified against the view. */
  entries: ReadonlyArray<ILevelViewerSnapshotEntry>;
  /** The level open, which the file's is checked against. */
  level: Nullable<string>;
  /** What the apply came to, shown in place of the choice once it ran; null before. */
  result: Nullable<ILevelViewerSnapshotApplied>;
  /** Whether the values chosen are being applied, during which they are not applied again. */
  isApplying?: boolean;
  onApply: (entries: Array<ILevelViewerSnapshotEntry>) => void;
  onClose: () => void;
}

/**
 * Imports a level view's snapshot in part: what it changes of the view by part, each chosen to apply; what it does not
 * apply and why; and, on asking, what stays as it is.
 */
export function LevelViewerImportDialog({
  "data-testid": dataTestId = "level-viewer-import-dialog",
  id,
  className,
  fileName,
  snapshot,
  entries,
  level,
  result,
  isApplying = false,
  onApply,
  onClose,
}: ILevelViewerImportDialogProps): ReactElement {
  const titleId: string = useId();
  // The values by what importing them does, and the changes by the part of the view each belongs to, in file order.
  const { byStatus, groups }: ILevelViewerImportSort = useMemo(() => {
    const sort: ILevelViewerImportSort = { byStatus: { change: [], same: [], skip: [] }, groups: new Map() };

    for (const entry of entries) {
      sort.byStatus[entry.status].push(entry);

      if (entry.status === ELevelViewerSnapshotStatus.CHANGE) {
        sort.groups.set(entry.group, [...(sort.groups.get(entry.group) ?? []), entry]);
      }
    }

    return sort;
  }, [entries]);
  const changes: Array<ILevelViewerSnapshotEntry> = byStatus[ELevelViewerSnapshotStatus.CHANGE];
  const skipped: Array<ILevelViewerSnapshotEntry> = byStatus[ELevelViewerSnapshotStatus.SKIP];
  const unchanged: Array<ILevelViewerSnapshotEntry> = byStatus[ELevelViewerSnapshotStatus.SAME];
  const [chosen, setChosen] = useState<ReadonlySet<string>>(
    () => new Set(changes.map((it: ILevelViewerSnapshotEntry) => it.key))
  );
  const [isUnchangedShown, setUnchangedShown] = useState<boolean>(false);
  const fileLevel: Nullable<string> = typeof snapshot.level === "string" ? snapshot.level : null;
  const exportedAt: number = typeof snapshot.exportedAt === "string" ? Date.parse(snapshot.exportedAt) : Number.NaN;
  const commit: Nullable<string> = typeof snapshot.build?.commit === "string" ? snapshot.build.commit : null;
  const facts: Array<string> = [
    fileLevel ?? "no level named",
    ...(commit ? [`build ${commit.slice(0, 9)}${snapshot.build?.isDirty ? " with local changes" : ""}`] : []),
    ...(Number.isFinite(exportedAt) ? [`exported ${formatInstant(exportedAt)}`] : []),
  ];

  const onToggle = useCallback(
    (keys: ReadonlyArray<string>, isChosen: boolean) =>
      setChosen((current: ReadonlySet<string>) => {
        const next: Set<string> = new Set(current);

        keys.forEach((key: string) => (isChosen ? next.add(key) : next.delete(key)));

        return next;
      }),
    []
  );

  const onApplyChosen = useCallback(
    () => onApply(entries.filter((it: ILevelViewerSnapshotEntry) => chosen.has(it.key))),
    [chosen, entries, onApply]
  );

  const onToggleUnchanged = useCallback(() => setUnchangedShown((isShown: boolean) => !isShown), []);

  return (
    <Dialog
      data-testid={dataTestId}
      id={id}
      className={className}
      aria-labelledby={titleId}
      open={true}
      maxWidth={"md"}
      fullWidth={true}
      onClose={onClose}
    >
      <DialogHeader
        title={"Import view"}
        titleId={titleId}
        icon={<FileOpenIcon fontSize={"small"} />}
        closeLabel={"Close the import"}
        onClose={onClose}
      />

      <DialogContent className={"flex max-h-dialog-height flex-col gap-4 px-dialog"}>
        <div className={"flex flex-col gap-0.5"}>
          <Typography className={"monospace truncate"} variant={"body2"} title={fileName}>
            {fileName}
          </Typography>
          <Typography className={"text-text-secondary"} variant={"caption"}>
            {facts.join(" · ")}
          </Typography>
        </div>

        {fileLevel && fileLevel !== level ? (
          <Alert severity={"warning"}>
            {`Taken on ${fileLevel} while ${level ?? "no level"} is open: the camera stays where it is.`}
          </Alert>
        ) : null}

        {result ? (
          <Alert severity={result.failed.length ? "warning" : "success"}>
            {`Applied ${formatCount(result.applied.length)} value(s).`}
            {result.failed.map((it: { key: string; reason: string }) => (
              <div key={it.key}>{`${it.key}: ${it.reason}`}</div>
            ))}
          </Alert>
        ) : (
          <>
            <DetailSection
              title={"Changes"}
              description={"What the file sets other than the view draws it now; untick what should stay."}
              fact={`${formatCount(chosen.size)} of ${formatCount(changes.length)} chosen`}
            >
              {groups.size ? (
                [...groups].map(([group, members]: [string, Array<ILevelViewerSnapshotEntry>]) => (
                  <LevelViewerImportGroup
                    key={group}
                    group={group}
                    entries={members}
                    chosen={chosen}
                    onToggle={onToggle}
                  />
                ))
              ) : (
                <Typography variant={"body2"}>The view already draws what the file holds.</Typography>
              )}
            </DetailSection>

            {skipped.length ? (
              <DetailSection
                title={"Not applied"}
                description={"Values this build does not take, the application's own settings, or not for here."}
                fact={formatCount(skipped.length)}
              >
                {skipped.map((entry: ILevelViewerSnapshotEntry) => (
                  <LevelViewerImportRow key={entry.key} entry={entry} />
                ))}
              </DetailSection>
            ) : null}

            <DetailSection
              title={"Unchanged"}
              description={"Values the view already draws as the file has them."}
              fact={formatCount(unchanged.length)}
              action={
                unchanged.length ? (
                  <Button size={"small"} onClick={onToggleUnchanged}>
                    {isUnchangedShown ? "Hide" : "Show"}
                  </Button>
                ) : null
              }
            >
              {isUnchangedShown
                ? unchanged.map((entry: ILevelViewerSnapshotEntry) => (
                    <LevelViewerImportRow key={entry.key} entry={entry} />
                  ))
                : null}
            </DetailSection>
          </>
        )}
      </DialogContent>

      <DialogActions>
        <Button autoFocus={true} size={"small"} onClick={onClose}>
          {result ? "Close" : "Cancel"}
        </Button>

        {result ? null : (
          <Button size={"small"} variant={"contained"} disabled={!chosen.size || isApplying} onClick={onApplyChosen}>
            {`Apply ${formatCount(chosen.size)}`}
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
}
