import { useInjection } from "@wirestate/react";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { Maybe, Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { formatLevelPoint } from "@/core/level/lib/camera/level-camera";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { describeLevelSurface, toLevelSurfaceSummary } from "@/core/level/lib/surface/level-surface-summary";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { RenderViewportOverlay } from "@/core/render/components/overlay";

/** The sector, the entry, and which mesh or impostor of the sector's it is. */
function toSurfaceHeading(picked: Extract<TLevelPick, { kind: ELevelPick.SURFACE }>): string {
  const { sector, shaderId, mesh, place, isImpostor } = picked;
  const drawn: string = isImpostor ? ` · impostor ${place}` : mesh === null ? "" : ` · mesh ${mesh}, place ${place}`;

  return `sector ${sector} · entry ${shaderId}${drawn}`;
}

/** The shader an entry names and its base texture, as the Surfaces panel names it. */
function toSurfaceNaming(surfaces: ReadonlyArray<XraySurfaceDescriptor>, shaderId: number): string {
  const descriptor: Maybe<XraySurfaceDescriptor> = surfaces[shaderId];

  return descriptor ? describeLevelSurface(toLevelSurfaceSummary(descriptor, shaderId)) : "";
}

/**
 * What the last click in the viewport picked, over the scene: the sector and shader table entry drawing a surface, or
 * the spawned object and the visual it stands as, and where the click met it.
 */
export function LevelPreviewPick({
  "data-testid": dataTestId = "level-preview-pick",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const loadService: LevelLoadService = useInjection(LevelLoadService);

  const picked: Nullable<TLevelPick> = viewportService.picked;
  const surfaces: Nullable<ReadonlyArray<XraySurfaceDescriptor>> = loadService.surfaces.value;

  if (!picked) {
    return <></>;
  }

  return (
    <RenderViewportOverlay data-testid={dataTestId} id={id} className={className} corner={"bottom-left"}>
      {picked.kind === ELevelPick.SPAWN ? (
        <>
          <div>{`${picked.object.name} · ${picked.object.section}`}</div>
          <div>{picked.visual}</div>
        </>
      ) : (
        <>
          <div>{toSurfaceHeading(picked)}</div>
          <div>{toSurfaceNaming(surfaces ?? [], picked.shaderId)}</div>
        </>
      )}
      <div>{formatLevelPoint(picked.point)}</div>
    </RenderViewportOverlay>
  );
}
