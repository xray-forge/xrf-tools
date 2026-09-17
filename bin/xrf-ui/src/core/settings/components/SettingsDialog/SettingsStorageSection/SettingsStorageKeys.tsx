import { Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { DetailSection } from "@/core/ui/layout/DetailSection";
import { formatBytes } from "@/lib/memory/format";

import { describeKeyCount, IStorageEntry } from "./SettingsStorageSection.utils";

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
    <DetailSection
      title={"Keys"}
      description={"Every key and what it occupies, largest first."}
      fact={describeKeyCount(entries.length)}
    >
      <Stack className={"mt-2"}>
        {listed.map((it: IStorageEntry) => (
          <div key={it.key} className={"flex gap-4 py-0.5"}>
            <Typography className={"monospace min-w-0 grow wrap-anywhere"}>{it.key}</Typography>

            <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
              {formatBytes(it.size)}
            </Typography>
          </div>
        ))}

        {entries.length > listed.length ? (
          <Typography className={"mt-2 text-text-secondary"} variant={"caption"}>
            {`${entries.length - listed.length} more not listed.`}
          </Typography>
        ) : null}
      </Stack>
    </DetailSection>
  );
}
