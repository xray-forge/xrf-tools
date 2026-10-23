import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatBytes, formatCount } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback } from "react";

import { LEVEL_CAPTURE_DDS_FORMATS } from "@/applications/level-viewer/lib/capture/level-capture";
import { ILevelViewerImage } from "@/applications/level-viewer/lib/files/level-viewer-image";
import {
  ILevelMapAnnotations,
  LEVEL_MAP_ANNOTATION_KINDS,
  toLevelMapAnnotationsWith,
} from "@/applications/level-viewer/lib/map/level-map-annotations";
import {
  ILevelMapExportSettings,
  isLevelMapExportKindOffered,
  LEVEL_MAP_BACKDROP,
  LEVEL_MAP_EXPORT_LONG_SIDES,
  toLevelMapExportSize,
} from "@/applications/level-viewer/lib/map/level-map-export";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { ELevelAnnotationKind, EMapExportAnnotations, EMapExportFormat } from "@/core/ipc/types/xrf-app";
import { ETextureEncodingFormat } from "@/core/ipc/types/xrf-texture";
import { cn } from "@/core/theme/class-name";
import { CheckboxFormRow, ChoiceFormRow } from "@/core/ui/form";
import { IImageViewportView, ImageViewport } from "@/core/ui/media/ImageViewport";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { LevelViewerCaptureExported } from "./LevelViewerCaptureExported";
import { LevelViewerMapAnnotationsOverlay } from "./LevelViewerMapAnnotationsOverlay";

/** How the annotations go into an export, keyed for a choice: none, over the map, or alone. */
const ANNOTATIONS_NONE: string = "none";

interface ILevelViewerMapResultProps extends BaseComponentProps {
  map: ILevelViewerImage;
}

/**
 * A captured level map: the map itself to pan and zoom over its coverage with its annotations over it, what of them is
 * drawn, its file and size, and how it is exported again: as a DDS the game loads, a PNG or the engine's own TGA, at a
 * size, its coverage kept or laid over black, its annotations over it, alone or not at all, alone also as their SVG.
 */
export function LevelViewerMapResult({
  "data-testid": dataTestId = "level-viewer-map-result",
  id,
  className,
  map,
}: ILevelViewerMapResultProps): ReactElement {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);
  const settings: ILevelMapExportSettings = captureService.mapExport;
  const captured: number = Math.max(map.width, map.height);
  const [width, height]: [number, number] = toLevelMapExportSize(
    map.width,
    map.height,
    settings.longSide,
    settings.kind
  );
  const isDds: boolean = settings.kind === EMapExportFormat.DDS;
  const isSvg: boolean = settings.kind === EMapExportFormat.SVG;
  const annotations: ILevelMapAnnotations = captureService.mapAnnotations;
  const annotationsUrl: Nullable<string> = captureService.annotationsUrl;

  const renderOverlay = useCallback(
    (view: IImageViewportView): ReactNode =>
      annotationsUrl ? <LevelViewerMapAnnotationsOverlay view={view} url={annotationsUrl} /> : null,
    [annotationsUrl]
  );

  const onAnnotationKind = useCallback(
    (kind: ELevelAnnotationKind, isShown: boolean) =>
      captureService.setMapAnnotations(toLevelMapAnnotationsWith(annotations, kind, isShown)),
    [annotations, captureService]
  );

  const onLabelled = useCallback(
    (isLabelled: boolean) => captureService.setMapAnnotations({ ...annotations, isLabelled }),
    [annotations, captureService]
  );

  const onExportAnnotations = useCallback(
    (value: string) =>
      captureService.setMapExport({
        annotations: value === ANNOTATIONS_NONE ? null : (value as EMapExportAnnotations),
      }),
    [captureService]
  );

  const onKind = useCallback((kind: EMapExportFormat) => captureService.setMapExport({ kind }), [captureService]);

  const onFormat = useCallback(
    (format: ETextureEncodingFormat) => captureService.setMapExport({ format }),
    [captureService]
  );

  const onLongSide = useCallback(
    (value: string) => captureService.setMapExport({ longSide: Number(value) }),
    [captureService]
  );

  const onCoverage = useCallback(
    (isKept: boolean) => captureService.setMapExport({ backdrop: isKept ? null : LEVEL_MAP_BACKDROP }),
    [captureService]
  );

  const onMipmapped = useCallback(
    (isMipmapped: boolean) => captureService.setMapExport({ isMipmapped }),
    [captureService]
  );

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 flex-col gap-3", className)}>
      <div className={"flex h-[48vh] min-h-0 flex-col overflow-hidden rounded-surface"}>
        <ImageViewport
          alt={`Level map ${map.name}`}
          src={map.url}
          width={map.width}
          height={map.height}
          renderOverlay={renderOverlay}
        />
      </div>

      <div className={"flex flex-col gap-0.5"}>
        <Typography className={"monospace truncate"} variant={"body2"} title={map.path}>
          {map.name}
        </Typography>
        <Typography className={"text-text-secondary"} variant={"caption"}>
          {`${formatCount(map.width)} × ${formatCount(map.height)} pixels · ${formatBytes(map.size)}`}
        </Typography>
      </div>

      <div className={"grid grid-cols-3 gap-x-6"}>
        {LEVEL_MAP_ANNOTATION_KINDS.map(([kind, label]) => (
          <CheckboxFormRow
            key={kind}
            label={label}
            isChecked={annotations.kinds.includes(kind)}
            onChange={(isShown: boolean) => onAnnotationKind(kind, isShown)}
          />
        ))}
        <CheckboxFormRow label={"Name every object"} isChecked={annotations.isLabelled} onChange={onLabelled} />
      </div>

      <div className={"grid grid-cols-2 gap-x-6 gap-y-3"}>
        <ChoiceFormRow
          label={"Annotations"}
          description={"Over the map, or alone and clear where they draw nothing, as the export's kind or as an SVG."}
          options={[
            { value: ANNOTATIONS_NONE, label: "None" },
            { value: EMapExportAnnotations.OVER, label: "Over the map" },
            { value: EMapExportAnnotations.ALONE, label: "Alone" },
          ]}
          value={settings.annotations ?? ANNOTATIONS_NONE}
          onChange={onExportAnnotations}
        />
        <ChoiceFormRow
          label={"Export as"}
          options={[
            { value: EMapExportFormat.DDS, label: "DDS" },
            { value: EMapExportFormat.PNG, label: "PNG" },
            { value: EMapExportFormat.TGA, label: "TGA" },
            {
              value: EMapExportFormat.SVG,
              label: "SVG",
              isDisabled: !isLevelMapExportKindOffered(EMapExportFormat.SVG, settings.annotations),
            },
          ]}
          value={settings.kind}
          onChange={onKind}
        />
        <ChoiceFormRow
          label={"Format"}
          description={"BC7 is the sharpest and loads on DX11; BC3 is what the shipped maps are."}
          options={LEVEL_CAPTURE_DDS_FORMATS}
          value={settings.format}
          isDisabled={!isDds}
          onChange={onFormat}
        />
        <ChoiceFormRow
          label={"Size"}
          options={LEVEL_MAP_EXPORT_LONG_SIDES.map((side: number) => ({
            value: String(side),
            label: formatCount(side),
            isDisabled: side > captured,
          }))}
          value={String(Math.min(settings.longSide, captured))}
          onChange={onLongSide}
        />
        <div className={"flex flex-col"}>
          <CheckboxFormRow label={"Keep coverage"} isChecked={settings.backdrop === null} onChange={onCoverage} />
          <CheckboxFormRow
            label={"Mipmaps"}
            isChecked={settings.isMipmapped}
            isDisabled={!isDds}
            onChange={onMipmapped}
          />
        </div>
      </div>

      <Typography className={"text-text-secondary"} variant={"caption"}>
        {isSvg
          ? "Exports the annotations as drawn, a vector over the capture's pixels"
          : `Exports at ${formatCount(width)} × ${formatCount(height)} pixels` +
            (isDds ? ", whole four pixel blocks covering the area exactly" : "")}
      </Typography>

      <LevelViewerCaptureExported />
    </div>
  );
}
