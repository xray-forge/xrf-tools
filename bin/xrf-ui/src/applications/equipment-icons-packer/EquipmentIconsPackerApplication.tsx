import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { EquipmentPackResult } from "@/applications/equipment-icons-packer/components/EquipmentPackResult";
import { EquipmentService, IPackEquipmentResult } from "@/core/equipment-icons";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import {
  getPathIfExists,
  getProjectEquipmentDDSPath,
  getProjectEquipmentSourcePath,
  getProjectSystemLtxPath,
} from "@/core/settings/lib/path";
import { ProjectService } from "@/core/settings/services/project";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function EquipmentIconsPackerApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const notify: TEmitNotification = useEmitNotification();

  const equipmentService: EquipmentService = useInjection(EquipmentService);
  const projectService: ProjectService = useInjection(ProjectService);

  const [result, setResult] = useState<Loadable<Nullable<IPackEquipmentResult>>>(() => createLoadable(null));

  // The source is the directory of loose icons and the output is the single dds built from them. The
  // dialogs used to be configured the other way round, so browsing either one offered the wrong kind of
  // thing entirely.
  const source: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_PACKER,
    id: "source",
    title: "Select source icons directory",
    isDirectory: true,
    isDisabled: result.isLoading,
    seed: async () =>
      projectService.xrfProjectPath
        ? getPathIfExists(getProjectEquipmentSourcePath(projectService.xrfProjectPath))
        : null,
  });

  const output: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_PACKER,
    id: "output",
    title: "Select output sprite",
    filters: [{ name: "dds", extensions: ["dds"] }],
    isSave: true,
    isDisabled: result.isLoading,
    seed: async () =>
      projectService.xrfProjectPath ? getProjectEquipmentDDSPath(projectService.xrfProjectPath) : null,
  });

  const systemLtx: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_PACKER,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: [{ name: "ltx", extensions: ["ltx"] }],
    isDisabled: result.isLoading,
    seed: async () =>
      projectService.xrfProjectPath ? getPathIfExists(getProjectSystemLtxPath(projectService.xrfProjectPath)) : null,
  });

  const onPackEquipmentClicked = useCallback(async () => {
    if (!source.value || !output.value || !systemLtx.value) {
      return log.info("Cannot pack equipment sprite without every path");
    }

    try {
      setResult(createLoadable(null, true));

      const packResult: IPackEquipmentResult = await equipmentService.packEquipmentSprite(
        source.value,
        output.value,
        systemLtx.value
      );

      setResult(createLoadable(packResult));

      notify({
        details: output.value,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.EQUIPMENT_ICONS_PACKER,
        title: "Packed equipment sprite",
      });
    } catch (error) {
      log.error("Failed to pack equipment-editor:", error);

      setResult(createLoadable(null, false, error instanceof Error ? error : new Error(String(error))));

      notify({
        details: `${output.value}\n${String(error)}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.EQUIPMENT_ICONS_PACKER,
        title: "Could not pack equipment sprite",
      });
    }
  }, [equipmentService, log, notify, output.value, source.value, systemLtx.value]);

  return (
    <PickerForm
      isLoading={result.isLoading}
      isSubmitDisabled={!source.isValid || !output.isValid || !systemLtx.isValid}
      title={"Pack equipment sprite"}
      description={"Builds one sprite from a directory of icons. The output file is overwritten."}
      error={result.error ? String(result.error) : undefined}
      submitLabel={"Pack"}
      result={result.value ? <EquipmentPackResult result={result.value} /> : null}
      onSubmit={onPackEquipmentClicked}
    >
      <PathFormRow
        isDisabled={result.isLoading}
        label={"Source"}
        description={"Directory of individual icon files to pack"}
        field={source}
      />

      <PathFormRow
        isDisabled={result.isLoading}
        label={"Output"}
        description={"The *.dds sprite to write"}
        field={output}
      />

      <PathFormRow
        isDisabled={result.isLoading}
        label={"System configuration"}
        description={"The system.ltx that names the icons"}
        field={systemLtx}
      />
    </PickerForm>
  );
}
