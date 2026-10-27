import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { ReactElement, useCallback } from "react";

import { EToolId } from "@/core/routing/tool";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { ExportsService } from "@/tools/exports-explorer/services/exports.service";

export function ExportsOpenForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const exportsService: ExportsService = useInjection(ExportsService);

  const isLoading: boolean = exportsService.project.isLoading;

  const project: IPathField = usePathField({
    tool: EToolId.EXPORTS_EXPLORER,
    id: "project",
    title: "Select script sources directory",
    isDirectory: true,
    isDisabled: isLoading,
    // Unseeded on purpose: a TypeScript source tree is not a path any other tool shares, so it is asked for here
    // rather than configured once for the one tool that reads it.
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
        label={"Script sources"}
        description={"Root of the TypeScript sources whose extern declarations are read"}
        field={project}
      />
    </PickerForm>
  );
}
