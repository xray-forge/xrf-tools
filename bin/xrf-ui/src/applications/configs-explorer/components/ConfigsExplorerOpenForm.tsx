import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import {
  DIALECT_IDS,
  DIALECT_OPTIONS,
  EConfigsDialect,
} from "@/applications/configs-explorer/components/ConfigsExplorerOpenForm.utils";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { ChoiceFormRow, IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

interface IConfigsExplorerOpenFormProps extends BaseComponentProps {
  /** Called once an open attempt has finished, successfully or not. */
  onFinished?: () => void;
}

/**
 * The way into the explorer: name a configs tree, and say which rules resolve it.
 */
export function ConfigsExplorerOpenForm({
  "data-testid": dataTestId = "configs-explorer-open-form",
  onFinished,
}: IConfigsExplorerOpenFormProps): ReactElement {
  const projectService: ConfigsProjectService = useInjection(ConfigsProjectService);

  const log: Logger = useLogger(__MODULE_NAME__);

  const root: IPathField = usePathField({
    application: EApplicationId.CONFIGS_EXPLORER,
    id: "root",
    title: "Select configs directory or game installation",
    isDirectory: true,
  });

  const [dialect, setDialect] = useRememberedValue<EConfigsDialect>({
    allowed: DIALECT_IDS,
    application: EApplicationId.CONFIGS_EXPLORER,
    fallback: EConfigsDialect.LTX,
    id: "dialect",
  });

  const isDltx: boolean = dialect === EConfigsDialect.DLTX;

  const onOpen = useCallback(async () => {
    if (!root.value) {
      return;
    }

    log.info("Opening configs project:", root.value, dialect);

    await projectService.open(root.value, isDltx);

    onFinished?.();
  }, [dialect, isDltx, log, onFinished, projectService, root.value]);

  return (
    <PickerForm
      data-testid={dataTestId}
      title={"Browse LTX configs"}
      description={"Reads the tree into a browsable project. Nothing is written."}
      submitLabel={"Open"}
      isSubmitDisabled={!root.isValid}
      isLoading={projectService.project.isLoading}
      error={projectService.project.error?.message}
      onSubmit={onOpen}
    >
      <PathFormRow
        label={"Configs root"}
        description={"A configs directory, or a game installation holding fsgame.ltx"}
        field={root}
      />

      <ChoiceFormRow
        label={"Dialect"}
        description={"DLTX applies the mod_*.ltx patch files a Monolith or Anomaly install carries"}
        options={DIALECT_OPTIONS}
        value={dialect}
        onChange={setDialect}
      />
    </PickerForm>
  );
}
