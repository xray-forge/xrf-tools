import { Alert, Stack } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveHeaderFields } from "@/core/archive/components/ArchiveHeaderFields";
import { HEADER_ENTRY_POINT, readHeaderValue } from "@/core/archive/lib";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IPackerHeaderSectionProps extends BaseComponentProps {
  config: ArchivePackConfig;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePackConfig>) => void;
}

/**
 * The header written into the archive, which is how the engine decides where its contents mount.
 */
export function PackerHeaderSection({
  "data-testid": dataTestId = "packer-header-section",
  id,
  className,
  config,
  isDisabled,
  onChange,
}: IPackerHeaderSectionProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);

  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
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
