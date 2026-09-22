import { Button, Divider, Stack, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useState } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { useLocalStorageRevision } from "@/lib/local-storage";
import { formatBytes } from "@/lib/memory/format";

import { SettingsStorageKeys } from "./SettingsStorageKeys";
import {
  clearStorageGroup,
  describeKeyCount,
  IStorageEntry,
  IStorageGroupUsage,
  IStorageUsage,
  measureLocalStorage,
  STORAGE_BUDGET_BYTES,
} from "./SettingsStorageSection.utils";

/** Every measured key across the groups, largest first. */
function everyEntry(usage: IStorageUsage): Array<IStorageEntry> {
  return usage.groups
    .flatMap((it: IStorageGroupUsage) => it.entries)
    .sort((left: IStorageEntry, right: IStorageEntry) => right.size - left.size);
}

/**
 * What the application has remembered, and what may be forgotten.
 */
export function SettingsStorageSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const usage: IStorageUsage = measureLocalStorage();

  /** The group a person asked to empty, held until they confirm it. */
  const [pending, setPending] = useState<Nullable<IStorageGroupUsage>>(null);

  const onConfirmClear = useCallback((): void => {
    if (pending) {
      clearStorageGroup(pending);
    }

    setPending(null);
  }, [pending]);

  useLocalStorageRevision();

  return (
    <div className={"flex flex-col gap-6"}>
      <DetailSection
        title={"Local storage"}
        description={
          "Everything the application remembers between sessions lives in the webview's own storage. Nothing here " +
          "leaves this machine."
        }
        fact={`${formatBytes(usage.total)} of ${formatBytes(STORAGE_BUDGET_BYTES)}`}
      >
        <Stack className={"mt-2"} divider={<Divider flexItem />}>
          {usage.groups.map((group: IStorageGroupUsage) => (
            <div key={group.descriptor.id} className={"flex items-center gap-4 py-2"}>
              <div className={"min-w-0 grow"}>
                <Typography variant={"body2"}>{group.descriptor.label}</Typography>

                <Typography className={"block text-text-secondary"} variant={"caption"}>
                  {group.descriptor.description}
                </Typography>
              </div>

              <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
                {describeKeyCount(group.entries.length)}
              </Typography>

              <Typography className={"w-18 shrink-0 text-right"} variant={"body2"}>
                {formatBytes(group.size)}
              </Typography>

              <div className={"w-18 shrink-0"}>
                {group.descriptor.isClearable && group.entries.length ? (
                  <Button color={"inherit"} size={"small"} onClick={() => setPending(group)}>
                    Clear
                  </Button>
                ) : null}
              </div>
            </div>
          ))}
        </Stack>
      </DetailSection>

      {settingsService.isDevModeEnabled ? <SettingsStorageKeys entries={everyEntry(usage)} /> : null}

      <ConfirmDialog
        isOpen={Boolean(pending)}
        isDestructive={true}
        title={`Clear ${pending ? pending.descriptor.label.toLowerCase() : "storage"}?`}
        description={
          pending
            ? `Removes ${describeKeyCount(pending.entries.length)}, ${formatBytes(pending.size)}. ` +
              "This cannot be undone."
            : ""
        }
        confirmLabel={"Clear"}
        onConfirm={onConfirmClear}
        onClose={() => setPending(null)}
      />
    </div>
  );
}
