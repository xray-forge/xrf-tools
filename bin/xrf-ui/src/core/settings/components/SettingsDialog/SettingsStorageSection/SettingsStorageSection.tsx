import { Box, Button, Divider, Stack, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { SettingsService } from "@/core/settings/services/settings";
import { ConfirmDialog } from "@/core/ui/dialog/ConfirmDialog";
import { useLocalStorageRevision } from "@/lib/local-storage";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

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

/** Room the size and clear columns keep, so the group rows line up whatever they hold. */
const ACTION_COLUMN_WIDTH: number = 72;

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
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
      <SettingsSection
        title={"Local storage"}
        description={
          "Everything the application remembers between sessions lives in the webview's own storage. Nothing here " +
          "leaves this machine."
        }
        fact={`${formatBytes(usage.total)} of ${formatBytes(STORAGE_BUDGET_BYTES)}`}
      >
        <Stack divider={<Divider flexItem />} sx={{ marginTop: 1 }}>
          {usage.groups.map((group: IStorageGroupUsage) => (
            <Box key={group.descriptor.id} sx={{ alignItems: "center", display: "flex", gap: 2, paddingY: 1 }}>
              <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                <Typography variant={"body2"}>{group.descriptor.label}</Typography>

                <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block" }}>
                  {group.descriptor.description}
                </Typography>
              </Box>

              <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
                {describeKeyCount(group.entries.length)}
              </Typography>

              <Typography variant={"body2"} sx={{ flexShrink: 0, minWidth: ACTION_COLUMN_WIDTH, textAlign: "right" }}>
                {formatBytes(group.size)}
              </Typography>

              <Box sx={{ flexShrink: 0, width: ACTION_COLUMN_WIDTH }}>
                {group.descriptor.isClearable && group.entries.length ? (
                  <Button color={"inherit"} size={"small"} onClick={() => setPending(group)}>
                    Clear
                  </Button>
                ) : null}
              </Box>
            </Box>
          ))}
        </Stack>
      </SettingsSection>

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
    </Box>
  );
}
