import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { EquipmentPackResult } from "@/applications/equipment-icons-packer/components/EquipmentPackResult";
import { EquipmentService, IPackEquipmentResult } from "@/core/equipment-icons";
import { JobProgressView } from "@/core/jobs/components/JobProgressView";
import { IJobState } from "@/core/jobs/lib";
import { EApplicationId } from "@/core/routing/application";
import { EPathRole, resolveExistingPathRole, resolvePathRole } from "@/core/settings/lib/path";
import { PathsService } from "@/core/settings/services/paths";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { PathFormRow } from "@/core/ui/form/PathFormRow";
import { IPathField, usePathField } from "@/core/ui/form/use-path-field";
import { createLoadable, Loadable } from "@/lib/loadable";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export function EquipmentIconsPackerApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const equipmentService: EquipmentService = useInjection(EquipmentService);

  const pathsService: PathsService = useInjection(PathsService);

  // The run rather than this view's own loadable: a pack survives the window being reloaded, so returning here finds
  // it again instead of offering a Pack button the lease would then refuse.
  const job: Nullable<IJobState> = equipmentService.packJob;

  const [result, setResult] = useState<Loadable<Nullable<IPackEquipmentResult>>>(() => createLoadable(null));

  const isRunning: boolean = Boolean(job) || result.isLoading;

  // The source is the directory of loose icons and the output is the single dds built from them. The
  // dialogs used to be configured the other way round, so browsing either one offered the wrong kind of
  // thing entirely.
  const source: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_PACKER,
    id: "source",
    title: "Select source icons directory",
    isDirectory: true,
    isDisabled: isRunning,
    seed: () => resolveExistingPathRole(EPathRole.EQUIPMENT_ICON_SOURCES, pathsService.paths),
  });

  const output: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_PACKER,
    id: "output",
    title: "Select output sprite",
    filters: [{ name: "dds", extensions: ["dds"] }],
    isSave: true,
    isDisabled: isRunning,
    seed: () => resolvePathRole(EPathRole.EQUIPMENT_SPRITE, pathsService.paths),
  });

  const systemLtx: IPathField = usePathField({
    application: EApplicationId.EQUIPMENT_ICONS_PACKER,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: [{ name: "ltx", extensions: ["ltx"] }],
    isDisabled: isRunning,
    seed: () => resolveExistingPathRole(EPathRole.SYSTEM_LTX, pathsService.paths),
  });

  const onPackEquipmentClicked = useCallback(async () => {
    if (!source.value || !output.value || !systemLtx.value) {
      return log.info("Cannot pack equipment sprite without every path");
    }

    try {
      setResult(createLoadable(null, true));

      const packed: IPackEquipmentResult = await equipmentService.packEquipmentSprite(
        source.value,
        output.value,
        systemLtx.value
      );

      setResult(createLoadable(packed));
    } catch (error) {
      log.error("Failed to pack equipment-editor:", error);

      setResult(createLoadable(null, false, error instanceof Error ? error : new Error(String(error))));
    }
  }, [equipmentService, log, output.value, source.value, systemLtx.value]);

  const onCancel = useCallback(() => equipmentService.cancelPackEquipmentSprite(), [equipmentService]);

  return (
    <PickerForm
      isLoading={isRunning}
      isSubmitDisabled={!source.isValid || !output.isValid || !systemLtx.isValid}
      title={"Pack equipment sprite"}
      description={"Builds one sprite from a directory of icons. The output file is overwritten."}
      error={result.error ? String(result.error) : undefined}
      submitLabel={"Pack"}
      status={job ? <JobProgressView job={job} onCancel={onCancel} /> : null}
      result={result.value ? <EquipmentPackResult result={result.value} /> : null}
      onSubmit={onPackEquipmentClicked}
    >
      <PathFormRow
        isDisabled={isRunning}
        label={"Source"}
        description={"Directory of individual icon files to pack"}
        field={source}
      />

      <PathFormRow isDisabled={isRunning} label={"Output"} description={"The *.dds sprite to write"} field={output} />

      <PathFormRow
        isDisabled={isRunning}
        label={"System configuration"}
        description={"The system.ltx that names the icons"}
        field={systemLtx}
      />
    </PickerForm>
  );
}
