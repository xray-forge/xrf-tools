import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatBytes, formatCount } from "@xrf/format";
import { ETextureEncodingFormat } from "@xrf/ipc/types/xrf-texture";
import { ReactElement, useCallback } from "react";

import { LEVEL_CAPTURE_DDS_FORMATS } from "@/applications/level-viewer/lib/capture/level-capture";
import { toLevelCubeFaceSize } from "@/applications/level-viewer/lib/cube/level-cube-capture";
import {
  ILevelCubeExportSettings,
  isLevelCubeExportSingle,
  LEVEL_CUBE_EXPORT_SIZES,
  TLevelCubeExportKind,
  toLevelCubeExportSize,
} from "@/applications/level-viewer/lib/cube/level-cube-export";
import { ILevelViewerImage } from "@/applications/level-viewer/lib/files/level-viewer-image";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/capture-result";
import { cn } from "@/core/theme/class-name";
import { CheckboxFormRow, ChoiceFormRow } from "@/core/ui/form";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { LevelViewerCaptureExported } from "./LevelViewerCaptureExported";

interface ILevelViewerCubeResultProps extends BaseComponentProps {
  cross: ILevelViewerImage;
}

/**
 * A captured cubemap: its cross to pan and zoom over, its file and size, and how it is exported again: one cube DDS,
 * or six TGAs or PNGs into a folder, its faces at a size.
 */
export function LevelViewerCubeResult({
  "data-testid": dataTestId = "level-viewer-cube-result",
  id,
  className,
  cross,
}: ILevelViewerCubeResultProps): ReactElement {
  const resultService: LevelViewerCaptureResultService = useInjection(LevelViewerCaptureResultService);
  const settings: ILevelCubeExportSettings = resultService.cubeExport;
  const captured: number = toLevelCubeFaceSize(cross.width);
  const size: number = toLevelCubeExportSize(settings, captured);
  const isDds: boolean = isLevelCubeExportSingle(settings.kind);

  const onKind = useCallback((kind: TLevelCubeExportKind) => resultService.setCubeExport({ kind }), [resultService]);

  const onFormat = useCallback(
    (format: ETextureEncodingFormat) => resultService.setCubeExport({ format }),
    [resultService]
  );

  const onSize = useCallback((value: string) => resultService.setCubeExport({ size: Number(value) }), [resultService]);

  const onMipmapped = useCallback(
    (isMipmapped: boolean) => resultService.setCubeExport({ isMipmapped }),
    [resultService]
  );

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 flex-col gap-3", className)}>
      <div className={"flex h-[48vh] min-h-0 flex-col overflow-hidden rounded-surface"}>
        <ImageViewport alt={`Cubemap ${cross.name}`} src={cross.url} width={cross.width} height={cross.height} />
      </div>

      <div className={"flex flex-col gap-0.5"}>
        <Typography className={"monospace truncate"} variant={"body2"} title={cross.path}>
          {cross.name}
        </Typography>
        <Typography className={"text-text-secondary"} variant={"caption"}>
          {`${formatCount(captured)} pixels a face · ${formatBytes(cross.size)}`}
        </Typography>
      </div>

      <div className={"grid grid-cols-2 gap-x-6 gap-y-3"}>
        <ChoiceFormRow
          label={"Export as"}
          description={"Six TGAs are named 1 to 6 as the engine writes its faces; six PNGs by face, as _px to _nz."}
          options={[
            { value: "dds", label: "Cube DDS" },
            { value: "tga", label: "Six TGAs" },
            { value: "png", label: "Six PNGs" },
          ]}
          value={settings.kind}
          onChange={onKind}
        />
        <ChoiceFormRow
          label={"Format"}
          description={"BC7 is the sharpest and loads on DX11; BC3 and BC1 load everywhere."}
          options={LEVEL_CAPTURE_DDS_FORMATS}
          value={settings.format}
          isDisabled={!isDds}
          onChange={onFormat}
        />
        <ChoiceFormRow
          label={"Face size"}
          description={"The smallest are the #small cubes a sky lights the level by."}
          options={LEVEL_CUBE_EXPORT_SIZES.map((side: number) => ({
            value: String(side),
            label: formatCount(side),
            isDisabled: side > captured,
          }))}
          value={String(size)}
          onChange={onSize}
        />
        <CheckboxFormRow
          label={"Mipmaps"}
          isChecked={settings.isMipmapped}
          isDisabled={!isDds}
          onChange={onMipmapped}
        />
      </div>

      <LevelViewerCaptureExported />
    </div>
  );
}
