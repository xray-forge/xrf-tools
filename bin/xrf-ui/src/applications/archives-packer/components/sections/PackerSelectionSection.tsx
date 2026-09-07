import { Alert, Stack } from "@mui/material";
import { ReactElement } from "react";

import { PackerDirectoryList } from "@/applications/archives-packer/components/controls/PackerDirectoryList";
import { PackerStringList } from "@/applications/archives-packer/components/controls/PackerStringList";
import { isWholeDirectory } from "@/applications/archives-packer/lib/pack-config";
import { ArchivePackConfig } from "@/core/bindings/types/xrf-pack";
import { FormRow } from "@/core/ui/form";

interface IPackerSelectionSectionProps {
  config: ArchivePackConfig;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePackConfig>) => void;
}

/**
 * What goes into the archive: the sections an xrCompress configuration carries.
 */
export function PackerSelectionSection({ config, isDisabled, onChange }: IPackerSelectionSectionProps): ReactElement {
  return (
    <Stack spacing={2}>
      {isWholeDirectory(config) ? (
        <Alert severity={"info"}>
          Nothing is selected, so the whole source directory is packed. Add a directory or a file to narrow it.
        </Alert>
      ) : null}

      <FormRow
        label={"Included directories"}
        description={"Directories to pack, relative to the source. Recursive directories take their children too"}
      >
        <PackerDirectoryList
          directories={config.includeDirectories}
          isDisabled={isDisabled}
          addLabel={"Add directory"}
          emptyLabel={"No directories listed."}
          recursiveLabel={"Recursive"}
          onChange={(includeDirectories) => onChange({ includeDirectories })}
        />
      </FormRow>

      <FormRow label={"Included files"} description={"Individual files to pack, named relative to the source"}>
        <PackerStringList
          values={config.includeFiles}
          isDisabled={isDisabled}
          addLabel={"Add file"}
          emptyLabel={"No files listed."}
          placeholder={"shaders.xr"}
          onChange={(includeFiles) => onChange({ includeFiles })}
        />
      </FormRow>

      <FormRow
        label={"Excluded directories"}
        description={
          "Directories to leave out. A prefix match drops everything beneath it, otherwise only the exact path"
        }
      >
        <PackerDirectoryList
          directories={config.excludeDirectories}
          isDisabled={isDisabled}
          addLabel={"Add exclusion"}
          emptyLabel={"No exclusions."}
          recursiveLabel={"Prefix"}
          onChange={(excludeDirectories) => onChange({ excludeDirectories })}
        />
      </FormRow>

      <FormRow label={"Excluded extensions"} description={"Patterns matched against a file extension, such as *.txt"}>
        <PackerStringList
          values={config.excludeExtensions}
          isDisabled={isDisabled}
          addLabel={"Add pattern"}
          emptyLabel={"No patterns."}
          placeholder={"*.txt"}
          onChange={(excludeExtensions) => onChange({ excludeExtensions })}
        />
      </FormRow>
    </Stack>
  );
}
