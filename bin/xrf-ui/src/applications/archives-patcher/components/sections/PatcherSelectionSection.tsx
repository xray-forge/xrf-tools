import { Stack } from "@mui/material";
import { ReactElement } from "react";

import { ArchivePatchConfig } from "@/core/bindings/types/xrf-pack";
import { StringListFormRow } from "@/core/ui/form";

interface IPatcherSelectionSectionProps {
  config: ArchivePatchConfig;
  isDisabled?: boolean;
  onChange: (patch: Partial<ArchivePatchConfig>) => void;
}

/**
 * What the comparison is allowed to look at.
 */
export function PatcherSelectionSection({ config, isDisabled, onChange }: IPatcherSelectionSectionProps): ReactElement {
  return (
    <Stack spacing={2}>
      <StringListFormRow
        label={"Only compare"}
        description={"Logical prefixes the comparison is restricted to, such as configs"}
        values={config.include}
        addLabel={"Add prefix"}
        emptyLabel={"The whole of both sides is compared."}
        placeholder={"configs"}
        isDisabled={isDisabled}
        onChange={(include: Array<string>) => onChange({ include })}
      />

      <StringListFormRow
        label={"Ignore"}
        description={"Logical prefixes dropped from the comparison, applied after the restriction above"}
        values={config.ignore}
        addLabel={"Add prefix"}
        emptyLabel={"Nothing is ignored."}
        placeholder={"configs\\text"}
        isDisabled={isDisabled}
        onChange={(ignore: Array<string>) => onChange({ ignore })}
      />

      <StringListFormRow
        label={"Exclude extensions"}
        description={"Extension patterns that keep a file out of the comparison, matched with the dot"}
        values={config.excludeExtensions}
        addLabel={"Add pattern"}
        emptyLabel={"Every extension is compared."}
        placeholder={"*.txt"}
        isDisabled={isDisabled}
        onChange={(excludeExtensions: Array<string>) => onChange({ excludeExtensions })}
      />
    </Stack>
  );
}
