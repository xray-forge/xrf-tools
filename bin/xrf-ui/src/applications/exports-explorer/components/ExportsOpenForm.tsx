import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { EApplicationId } from "@/core/routing/application";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { Logger, useLogger } from "@/lib/logging";

export function ExportsOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const exportsService: ExportsService = useInjection(ExportsService);
  const projectService: ProjectService = useInjection(ProjectService);

  const isLoading: boolean = exportsService.project.isLoading;

  const project: IPathField = usePathField({
    application: EApplicationId.EXPORTS_EXPLORER,
    id: "project",
    title: "Select project directory",
    isDirectory: true,
    isDisabled: isLoading,
    seed: async () => projectService.xrfProjectPath ?? projectService.getXrfProjectPath(),
  });

  const onOpen = useCallback(() => {
    if (project.value) {
      void exportsService.openExportsProject(project.value);
    } else {
      log.info("Cannot open exports without a project path");
    }
  }, [exportsService, log, project.value]);

  return (
    <PickerForm
      isLoading={isLoading}
      isSubmitDisabled={!project.isValid}
      title={"Open script exports"}
      description={"Reads the project's TypeScript extern declarations. Nothing is written."}
      error={exportsService.project.error ? exportsService.project.error.message : undefined}
      submitLabel={"Open exports"}
      onSubmit={onOpen}
    >
      <PathFormRow
        isDisabled={isLoading}
        label={"Project"}
        description={"Root of the xrf project whose script exports are read"}
        field={project}
      />
    </PickerForm>
  );
}
