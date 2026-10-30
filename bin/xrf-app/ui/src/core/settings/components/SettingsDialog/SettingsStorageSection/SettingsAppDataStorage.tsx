import { formatBytes, formatCount } from "@xrf/format";
import { systemCommands } from "@xrf/ipc/commands/system";
import { AppStorageArea, AppStorageUsage, EAppStorageKind } from "@xrf/ipc/types/xrf-app-system";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Button } from "@xrf/ui/material/button";
import { Divider } from "@xrf/ui/material/divider";
import { Stack } from "@xrf/ui/material/stack";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback, useEffect, useState } from "react";

/** What each kind the application keeps on disk is, as the section says it. */
const APP_STORAGE_KINDS: Record<EAppStorageKind, { label: string; description: string }> = {
  [EAppStorageKind.RENDERER_CACHE]: {
    description:
      "What the renderer prepares once and reuses, so it starts and opens levels faster. Rebuilt when needed.",
    label: "Renderer cache",
  },
  [EAppStorageKind.OTHER_CACHES]: {
    description: "Every other cache, rebuilt when needed.",
    label: "Other caches",
  },
  [EAppStorageKind.LOGS]: {
    description: "This run's log and the last run's.",
    label: "Logs",
  },
  [EAppStorageKind.PREFERENCES]: {
    description: "The backend's own preferences.",
    label: "Preferences",
  },
  [EAppStorageKind.OTHER]: {
    description: "Anything else kept in the application's folder.",
    label: "Other",
  },
};

/** The kinds this section offers to clear: the ones rebuilt without asking anything of a person. */
const CLEARABLE_KINDS: ReadonlyArray<EAppStorageKind> = [EAppStorageKind.RENDERER_CACHE];

/**
 * @param files - How many files there are.
 * @returns The count with its noun agreeing.
 */
function describeFileCount(files: number): string {
  return files === 1 ? "1 file" : `${formatCount(files)} files`;
}

/**
 * What the application keeps in its own folder on disk, by kind, measured when the section opens, and clearing what is
 * rebuilt on its own.
 */
export function SettingsAppDataStorage(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [usage, setUsage] = useState<Nullable<AppStorageUsage>>(null);
  const [failure, setFailure] = useState<Nullable<string>>(null);
  /** The area a person asked to clear, held until they confirm it. */
  const [pending, setPending] = useState<Nullable<AppStorageArea>>(null);
  /** Bumped by a clear, so the folder is measured again. */
  const [revision, setRevision] = useState<number>(0);

  useEffect(() => {
    let isCurrent: boolean = true;

    systemCommands.getAppStorage().then(
      (measured: AppStorageUsage) => {
        if (isCurrent) {
          setUsage(measured);
          setFailure(null);
        }
      },
      (error: unknown) => {
        log.error("Failed to measure the application's storage:", error);

        if (isCurrent) {
          setFailure(String(error));
        }
      }
    );

    return () => {
      isCurrent = false;
    };
  }, [log, revision]);

  const onConfirmClear = useCallback(async (): Promise<void> => {
    const area: Nullable<AppStorageArea> = pending;

    setPending(null);

    if (!area) {
      return;
    }

    try {
      await systemCommands.clearAppStorage(area.kind as EAppStorageKind);
    } catch (error: unknown) {
      log.error("Failed to clear the application's storage:", area.kind, error);
      setFailure(String(error));
    }

    setRevision((it: number) => it + 1);
  }, [log, pending]);

  return (
    <DetailSection
      title={"Application data"}
      description={
        usage
          ? `What the application keeps on disk, in ${usage.root}.`
          : "What the application keeps on disk, in its own folder."
      }
      fact={usage ? formatBytes(usage.bytes) : "Measuring…"}
    >
      {failure ? (
        <Typography className={"mt-2 block text-error"} variant={"caption"}>
          {failure}
        </Typography>
      ) : null}

      <Stack className={"mt-2"} divider={<Divider flexItem />}>
        {(usage?.areas ?? []).map((area: AppStorageArea) => (
          <div key={area.kind} className={"flex items-center gap-4 py-2"}>
            <div className={"min-w-0 grow"}>
              <Typography variant={"body2"}>{APP_STORAGE_KINDS[area.kind as EAppStorageKind].label}</Typography>

              <Typography className={"block text-text-secondary"} variant={"caption"}>
                {APP_STORAGE_KINDS[area.kind as EAppStorageKind].description}
              </Typography>
            </div>

            <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
              {describeFileCount(area.files)}
            </Typography>

            <Typography className={"w-18 shrink-0 text-right"} variant={"body2"}>
              {formatBytes(area.bytes)}
            </Typography>

            <div className={"w-18 shrink-0"}>
              {CLEARABLE_KINDS.includes(area.kind as EAppStorageKind) && area.files ? (
                <Button color={"inherit"} size={"small"} onClick={() => setPending(area)}>
                  Clear
                </Button>
              ) : null}
            </div>
          </div>
        ))}
      </Stack>

      <ConfirmDialog
        isOpen={Boolean(pending)}
        isDestructive={true}
        title={`Clear ${pending ? APP_STORAGE_KINDS[pending.kind as EAppStorageKind].label.toLowerCase() : "storage"}?`}
        description={
          pending
            ? `Removes ${describeFileCount(pending.files)}, ${formatBytes(pending.bytes)}. They are rebuilt when needed.`
            : ""
        }
        confirmLabel={"Clear"}
        onConfirm={() => void onConfirmClear()}
        onClose={() => setPending(null)}
      />
    </DetailSection>
  );
}
