import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { ReactElement, useCallback } from "react";

import { ConfigsProjectService } from "@/applications/configs-explorer/services/configs-project";
import { ConfigsDialectFormRow } from "@/core/ltx/components/ConfigsDialectFormRow";
import { DIALECT_IDS, EConfigsDialect } from "@/core/ltx/lib/dialect";
import { EApplicationId } from "@/core/routing/application";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { IPathField, PathFormRow, usePathField, useRememberedValue } from "@/core/ui/form";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * The way into the explorer: name a configs tree, and say which rules resolve it.
 */
export function ConfigsExplorerOpenForm({
  "data-testid": dataTestId = "configs-explorer-open-form",
}: BaseComponentProps): ReactElement {
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
  }, [dialect, isDltx, log, projectService, root.value]);

  const onDialectChange = useCallback(
    (isDltx: boolean) => setDialect(isDltx ? EConfigsDialect.DLTX : EConfigsDialect.LTX),
    [setDialect]
  );

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
        description={
          "A configs directory, or a game installation holding fsgame.ltx (fsgame_<game>.ltx in an Enhanced Edition)"
        }
        field={root}
      />

      <ConfigsDialectFormRow isDltx={isDltx} isDisabled={projectService.project.isLoading} onChange={onDialectChange} />
    </PickerForm>
  );
}
