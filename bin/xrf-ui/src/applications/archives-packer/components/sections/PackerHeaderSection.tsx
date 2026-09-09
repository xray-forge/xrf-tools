import { Alert, Stack } from "@mui/material";
import { ReactElement } from "react";

import { HEADER_ENTRY_POINT, readHeaderValue } from "@/core/archive";
import { ArchiveHeaderFields } from "@/core/archive/components/ArchiveHeaderFields";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { Nullable } from "@/lib/types/general";

interface IPackerHeaderSectionProps {
  config: ArchivePackConfig;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePackConfig>) => void;
}

/**
 * The header written into the archive, which is how the engine decides where its contents mount.
 */
export function PackerHeaderSection({ config, isDisabled, onChange }: IPackerHeaderSectionProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);

  return (
    <Stack spacing={2}>
      {entryPoint ? null : (
        <Alert severity={"warning"}>
          Without an entry point the engine treats these volumes as encrypted Shadow of Chernobyl archives. Set one, or
          switch the extension to xdb under Options.
        </Alert>
      )}

      <ArchiveHeaderFields
        id={"packer"}
        header={config.header}
        entryPointDescription={"Where the engine mounts the contents. A packed gamedata tree wants the default"}
        isDisabled={isDisabled}
        onChange={(header) => onChange({ header })}
      />
    </Stack>
  );
}
