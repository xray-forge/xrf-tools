import { Box, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { describeKeyCount, IStorageEntry } from "@/core/settings/lib/storage-usage";
import { MONOSPACE } from "@/core/theme/tokens";
import { formatBytes } from "@/lib/memory/format";

/** How many keys are listed before it stops, so a store full of one field's history cannot fill the dialog. */
const KEY_LIMIT: number = 256;

interface ISettingsStorageKeysProps {
  entries: ReadonlyArray<IStorageEntry>;
}

/**
 * Every stored key and what it occupies, largest first.
 */
export function SettingsStorageKeys({ entries }: ISettingsStorageKeysProps): ReactElement {
  const listed: ReadonlyArray<IStorageEntry> = entries.slice(0, KEY_LIMIT);

  return (
    <SettingsSection
      title={"Keys"}
      description={"Every key and what it occupies, largest first."}
      fact={describeKeyCount(entries.length)}
    >
      <Stack sx={{ marginTop: 1 }}>
        {listed.map((it: IStorageEntry) => (
          <Box key={it.key} sx={{ display: "flex", gap: 2, paddingY: 0.25 }}>
            <Typography sx={{ ...MONOSPACE, flexGrow: 1, minWidth: 0, overflowWrap: "anywhere" }}>{it.key}</Typography>

            <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
              {formatBytes(it.size)}
            </Typography>
          </Box>
        ))}

        {entries.length > listed.length ? (
          <Typography variant={"caption"} sx={{ color: "text.secondary", marginTop: 1 }}>
            {`${entries.length - listed.length} more not listed.`}
          </Typography>
        ) : null}
      </Stack>
    </SettingsSection>
  );
}
