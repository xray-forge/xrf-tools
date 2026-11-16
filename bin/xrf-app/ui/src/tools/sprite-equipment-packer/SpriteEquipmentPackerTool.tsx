import { flowResult } from "@wirestate/mobx";
import { useInjection } from "@wirestate/react";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { ConfigsDialectFormRow } from "@/core/ltx/components/ConfigsDialectFormRow";
import { DDS_FILE_FILTERS, LTX_FILE_FILTERS } from "@/core/path/file-filters";
import { EToolId } from "@/core/routing/tool";
import { JobPickerForm } from "@/core/shell/chrome/JobPickerForm";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/sprite-equipment-packer.service";
import { IPathField, PathFormRow, usePathField } from "@/core/ui/form";

import { EquipmentPackResult } from "./components/EquipmentPackResult";

interface IEquipmentPackRequest {
  /** The directory of loose icons. */
  source: string;
  /** The sprite built from them. */
  output: string;
  systemLtx: string;
  isDltx: boolean;
}

export function SpriteEquipmentPackerTool(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const packerService: SpriteEquipmentPackerService = useInjection(SpriteEquipmentPackerService);

  // Rediscover a running pack after reload so the form cannot offer a second pack against the same output.
  const isRunning: boolean = packerService.operation.isRunning;

  // The source is the directory of loose icons and the output is the single dds built from them. The
  // dialogs used to be configured the other way round, so browsing either one offered the wrong kind of
  // thing entirely.
  const source: IPathField = usePathField({
    tool: EToolId.SPRITE_EQUIPMENT_PACKER,
    id: "source",
    title: "Select source icons directory",
    isDirectory: true,
    isDisabled: isRunning,
  });

  const output: IPathField = usePathField({
    tool: EToolId.SPRITE_EQUIPMENT_PACKER,
    id: "output",
    title: "Select output sprite",
    filters: DDS_FILE_FILTERS,
    isSave: true,
    isDisabled: isRunning,
  });

  const systemLtx: IPathField = usePathField({
    tool: EToolId.SPRITE_EQUIPMENT_PACKER,
    id: "system-ltx",
    title: "Select system.ltx",
    filters: LTX_FILE_FILTERS,
    isDisabled: isRunning,
  });

  // Opt-in rather than detected: a patched Anomaly tree and a vanilla one look alike, and resolving one under the
  // other's rules draws a sheet from wrong icon descriptors rather than failing.
  const [isDltx, setDltx] = useState<boolean>(false);

  const request: Nullable<IEquipmentPackRequest> = useMemo(() => {
    if (!source.value || !output.value || !systemLtx.value) {
      return null;
    }

    return { source: source.value, output: output.value, systemLtx: systemLtx.value, isDltx };
  }, [isDltx, output.value, source.value, systemLtx.value]);

  const onPackEquipment = useCallback(
    async (submitted: IEquipmentPackRequest) => {
      try {
        await flowResult(
          packerService.packEquipmentSprite(submitted.source, submitted.output, submitted.systemLtx, submitted.isDltx)
        );
      } catch (error) {
        log.error("Failed to pack equipment-editor:", error);
      }
    },
    [packerService, log]
  );

  return (
    <JobPickerForm
      operation={packerService.operation}
      request={request}
      isSubmitDisabled={!source.isValid || !output.isValid || !systemLtx.isValid}
      title={"Pack equipment sprite"}
      description={"Builds one sprite from a directory of icons. The output file is overwritten."}
      submitLabel={"Pack"}
      renderResult={(result) => <EquipmentPackResult result={result} />}
      onSubmit={onPackEquipment}
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
