import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";

export function ConfigsExplorerApplication(): ReactElement {
  const pathsService: PathsService = useInjection(PathsService);

  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_EXPLORER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
    seed: () => resolvePathRole(EPathRole.CONFIGS, pathsService.paths),
  });

  return (
    <PickerForm
      title={"Browse LTX configs"}
      description={"Reads the directory into a browsable tree. Nothing is written."}
      submitLabel={"Open"}
      isSubmitDisabled
    >
      <PathFormRow label={"Configs directory"} description={"Directory of LTX files to browse"} field={configs} />
    </PickerForm>
  );
}
