import { useInjection } from "@wirestate/react";
import { Maybe, Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { formatLevelPoint } from "@/core/level/lib/camera/level-camera";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { UNNAMED_LEVEL_SURFACE } from "@/core/level/lib/surface/level-surface-summary";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";
import { RenderViewportOverlay } from "@/core/render/components/overlay";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** The sector, the entry, and which mesh or impostor of the sector's it is. */
function toSurfaceHeading(picked: Extract<TLevelPick, { kind: ELevelPick.SURFACE }>): string {
  const { sector, shaderId, mesh, place, isImpostor } = picked;
  const drawn: string = isImpostor ? ` · impostor ${place}` : mesh === null ? "" : ` · mesh ${mesh}, place ${place}`;

  return `sector ${sector} · entry ${shaderId}${drawn}`;
}

/** The shader an entry names and its base texture, as the Surfaces panel names it. */
function toSurfaceNaming(surfaces: ReadonlyArray<XraySurfaceDescriptor>, shaderId: number): string {
  const descriptor: Maybe<XraySurfaceDescriptor> = surfaces[shaderId];

  return descriptor ? `${descriptor.shader ?? UNNAMED_LEVEL_SURFACE} / ${descriptor.textures[0] ?? "no texture"}` : "";
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
  const surfaces: Maybe<ReadonlyArray<XraySurfaceDescriptor>> = loadService.level.value?.selected.value.surfaces;

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
