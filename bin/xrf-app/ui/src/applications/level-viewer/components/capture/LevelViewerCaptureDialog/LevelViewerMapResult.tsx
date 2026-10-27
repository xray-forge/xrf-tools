import { useInjection } from "@wirestate/react";
import { formatBytes, formatCount } from "@xrf/format";
import { ELevelAnnotationKind, EMapExportAnnotations } from "@xrf/ipc/types/xrf-app-levels";
import { ETextureEncodingFormat } from "@xrf/ipc/types/xrf-texture";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material";
import { cn } from "@xrf/ui/theme/class-name";
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
  TLevelMapExportKind,
  toLevelMapExportLongSide,
  toLevelMapExportSize,
} from "@/applications/level-viewer/lib/map/level-map-export";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/level-viewer-capture-result";
import { CheckboxFormRow, ChoiceFormRow } from "@/core/ui/form";
import { IImageViewportView, ImageViewport } from "@/core/ui/media/ImageViewport";

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
  const resultService: LevelViewerCaptureResultService = useInjection(LevelViewerCaptureResultService);
  const settings: ILevelMapExportSettings = resultService.mapExport;
  const captured: number = Math.max(map.width, map.height);
  const [width, height]: [number, number] = toLevelMapExportSize(
    map.width,
    map.height,
    settings.longSide,
    settings.kind
  );
  const isDds: boolean = settings.kind === "dds";
  const isSvg: boolean = settings.kind === "svg";
  const annotations: ILevelMapAnnotations = resultService.mapAnnotations;
  const annotationsUrl: Nullable<string> = resultService.annotationsUrl;

  const renderOverlay = useCallback(
    (view: IImageViewportView): ReactNode =>
      annotationsUrl ? <LevelViewerMapAnnotationsOverlay view={view} url={annotationsUrl} /> : null,
    [annotationsUrl]
  );

  const onAnnotationKind = useCallback(
    (kind: ELevelAnnotationKind, isShown: boolean) =>
      resultService.setMapAnnotations(toLevelMapAnnotationsWith(annotations, kind, isShown)),
    [annotations, resultService]
  );

  const onLabelled = useCallback(
    (isLabelled: boolean) => resultService.setMapAnnotations({ ...annotations, isLabelled }),
    [annotations, resultService]
  );

  const onExportAnnotations = useCallback(
    (value: string) =>
      resultService.setMapExport({
        annotations: value === ANNOTATIONS_NONE ? null : (value as EMapExportAnnotations),
      }),
    [resultService]
  );

  const onKind = useCallback((kind: TLevelMapExportKind) => resultService.setMapExport({ kind }), [resultService]);

  const onFormat = useCallback(
    (format: ETextureEncodingFormat) => resultService.setMapExport({ format }),
    [resultService]
  );

  const onLongSide = useCallback(
    (value: string) => resultService.setMapExport({ longSide: Number(value) }),
    [resultService]
  );

  const onCoverage = useCallback(
    (isKept: boolean) => resultService.setMapExport({ backdrop: isKept ? null : LEVEL_MAP_BACKDROP }),
    [resultService]
  );

  const onMipmapped = useCallback(
    (isMipmapped: boolean) => resultService.setMapExport({ isMipmapped }),
    [resultService]
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
            { value: "dds", label: "DDS" },
            { value: "png", label: "PNG" },
            { value: "tga", label: "TGA" },
            {
              value: "svg",
              label: "SVG",
              isDisabled: !isLevelMapExportKindOffered("svg", settings.annotations),
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
          value={String(toLevelMapExportLongSide(settings, captured))}
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
