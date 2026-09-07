import { ReactElement } from "react";

import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";

export function ConfigsExplorerApplication(): ReactElement {
  const configs: IPathField = usePathField({
    application: EApplicationId.CONFIGS_EXPLORER,
    id: "directory",
    title: "Select configs directory",
    isDirectory: true,
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
