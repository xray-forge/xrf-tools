import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { ILevelCamera } from "@/core/level/lib/level-camera";
import { ILevelStats } from "@/core/level/lib/level-stats";
import { LevelViewportService } from "@/core/level/services";
import { useEditorStatus } from "@/core/shell/editor-shell";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

/**
 * Where the camera is, in the coordinates the level's own data is written in.
 */
export function formatLevelPosition(camera: ILevelCamera): string {
  const { position } = camera;

  return `x ${position.x.toFixed(1)} y ${position.y.toFixed(1)} z ${position.z.toFixed(1)}`;
}

/**
 * Which way the camera faces, as the engine states a direction.
 */
export function formatLevelFacing(camera: ILevelCamera): string {
  return `h ${toDegrees(camera.heading).toFixed(1)}° p ${toDegrees(camera.pitch).toFixed(1)}°`;
}

function toDegrees(radians: number): number {
  return (radians * 180) / Math.PI;
}

interface ILevelPreviewStatusProps {
  /** What the viewer is doing, when that is worth saying instead of what it is drawing. */
  activity?: Nullable<string>;
}

/**
 * Publishes what the level viewer is drawing to the application status bar.
 */
export function LevelPreviewStatus({ activity = null }: ILevelPreviewStatusProps): ReactElement {
  const viewport: LevelViewportService = useInjection(LevelViewportService);
  const camera: Nullable<ILevelCamera> = viewport.camera;
  const stats: ILevelStats = viewport.stats;

  const segments: Array<string> = useMemo(
    () =>
      activity
        ? [activity]
        : [
            ...(camera ? [formatLevelPosition(camera), formatLevelFacing(camera)] : []),
            `${stats.sectors} sectors`,
            `${stats.draws} draws`,
            `${stats.triangles} triangles`,
            formatBytes(stats.bytes),
          ],
    [activity, camera, stats]
  );

  useEditorStatus(segments);

  // JSX is needed to wrap with `observer` automatically, `null` kills reactivity.
  return <></>;
}
