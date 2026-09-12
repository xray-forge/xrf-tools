import { Alert, Stack } from "@mui/material";
import { ReactElement } from "react";

import { DEFAULT_ENTRY_POINT, HEADER_ENTRY_POINT, readHeaderValue } from "@/core/archive";
import { ArchiveHeaderFields } from "@/core/archive/components/ArchiveHeaderFields";
import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IPatcherHeaderSectionProps extends BaseComponentProps {
  config: ArchivePatchConfig;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * The header written into the patch volumes, which is how the engine decides where they mount.
 */
export function PatcherHeaderSection({
  "data-testid": dataTestId = "patcher-header-section",
  id,
  className,
  config,
  isDisabled,
  onChange,
}: IPatcherHeaderSectionProps): ReactElement {
  const entryPoint: Nullable<string> = readHeaderValue(config.header, HEADER_ENTRY_POINT);

  return (
    <Stack data-testid={dataTestId} id={id} className={className} spacing={2}>
      {entryPoint ? null : (
        <Alert severity={"warning"}>
          The engine reads the entry point without checking whether it is there, so volumes without one stop the game on
          load. Publishing refuses a header that omits it.
        </Alert>
      )}

      {entryPoint && entryPoint !== DEFAULT_ENTRY_POINT ? (
        <Alert severity={"info"} variant={"outlined"}>
          A patch overrides entries by name, so it has to mount where the release it patches did. Every shipped
          configuration uses {DEFAULT_ENTRY_POINT}.
        </Alert>
      ) : null}

      <ArchiveHeaderFields
        id={"patcher"}
        header={config.header}
        entryPointDescription={
          "Where the engine mounts the contents. A patch over a gamedata release wants the default"
        }
        isDisabled={isDisabled}
        onChange={(header) => onChange({ header })}
      />
    </Stack>
  );
}
