import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useState } from "react";

import { JobPickerForm } from "@/core/jobs/components/JobPickerForm";
import { ConfigsDialectFormRow } from "@/core/ltx/components/configs-dialect/ConfigsDialectFormRow";
import { DDS_FILE_FILTERS, LTX_FILE_FILTERS } from "@/core/path/file-filters";
import { EApplicationId } from "@/core/routing/application";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";
import { Logger, useLogger } from "@/lib/logging";

import { EquipmentPackResult } from "./components/EquipmentPackResult";

export function SpriteEquipmentPackerApplication(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const packerService: SpriteEquipmentPackerService = useInjection(SpriteEquipmentPackerService);

  // Rediscover a running pack after reload so the form cannot offer a second pack against the same output.
  const isRunning: boolean = packerService.operation.isRunning;

  // The source is the directory of loose icons and the output is the single dds built from them. The
  // dialogs used to be configured the other way round, so browsing either one offered the wrong kind of
  // thing entirely.
  const source: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_PACKER,
    id: "source",
    title: "Select source icons directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const output: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_PACKER,
    id: "output",
    title: "Select output sprite",
    filters: DDS_FILE_FILTERS,
    isSave: true,
    isDisabled: isRunning,
  });

  const systemLtx: IPathField = usePathField({
    application: EApplicationId.SPRITE_EQUIPMENT_PACKER,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: LTX_FILE_FILTERS,
    isDisabled: isRunning,
  });

  // Opt-in rather than detected: a patched Anomaly tree and a vanilla one look alike, and resolving one under the
  // other's rules draws a sheet from wrong icon descriptors rather than failing.
  const [isDltx, setDltx] = useState<boolean>(false);

  const onPackEquipmentClicked = useCallback(async () => {
    if (!source.value || !output.value || !systemLtx.value) {
      return log.info("Cannot pack equipment sprite without every path");
    }

    try {
      await flowResult(packerService.packEquipmentSprite(source.value, output.value, systemLtx.value, isDltx));
    } catch (error) {
      log.error("Failed to pack equipment-editor:", error);
    }
  }, [packerService, log, output.value, source.value, systemLtx.value, isDltx]);

  useEffect(() => {
    packerService.operation.reset();
  }, [source.value, output.value, systemLtx.value, isDltx, packerService]);

  return (
    <JobPickerForm
      operation={packerService.operation}
      isSubmitDisabled={!source.isValid || !output.isValid || !systemLtx.isValid}
      title={"Pack equipment sprite"}
      description={"Builds one sprite from a directory of icons. The output file is overwritten."}
      submitLabel={"Pack"}
      renderResult={(result) => <EquipmentPackResult result={result} />}
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

      <ConfigsDialectFormRow isDltx={isDltx} isDisabled={isRunning} onChange={setDltx} />
    </JobPickerForm>
  );
}
