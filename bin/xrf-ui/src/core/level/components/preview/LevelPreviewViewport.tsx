import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useRef } from "react";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { LevelPreviewScene } from "@/core/level/lib/scene";
import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { SettingsService } from "@/core/settings/services/settings";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface ILevelPreviewViewportProps extends BaseComponentProps {
  /** Resident sectors, as the loader publishes them. */
  sectors: ReadonlyMap<number, ILoadedSector>;
  /** The level's extent: what the grid is sized against, and where the camera opens. */
  bounds: Nullable<VisualBounds>;
  /** Where surfaces take their textures from, owned by the loader rather than by the scene. */
  textures?: Nullable<ILevelTextureLookup>;
  /** Counts changes to that set, which keeps one identity for the life of a level. */
  textureRevision?: number;
  options?: ILevelViewOptions;
  /** What the camera sees and how it answers input. */
  camera?: ILevelCameraOptions;
  /** What the viewer is lighting with, which is the viewer's answer rather than anything the level carries. */
  lighting?: ILevelLighting;
  /** Where the camera has gone, for the loader to stream against. */
  onCameraMoved: (point: ILevelPoint) => void;
  /** What the viewport costs and where its camera is, a few times a second while a level is open. */
  onReport?: (stats: ILevelStats, camera: ILevelCamera) => void;
}

/**
 * Mounts the imperative level scene and disposes it on unmount.
 */
export function LevelPreviewViewport({
  "data-testid": dataTestId = "level-preview-viewport",
  id,
  className,
  sectors,
  bounds,
  textures = null,
  textureRevision = 0,
  options = DEFAULT_LEVEL_VIEW_OPTIONS,
  camera = DEFAULT_LEVEL_CAMERA_OPTIONS,
  lighting = DEFAULT_LEVEL_LIGHTING,
  onCameraMoved,
  onReport,
}: ILevelPreviewViewportProps): ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);
  const settingsService: SettingsService = useInjection(SettingsService);
  const sceneRef = useRef<Nullable<LevelPreviewScene>>(null);

  // Held in refs so the scene is built once: rebuilding it because a handler identity changed would drop the webgl
  // context and everything uploaded into it.
  const cameraRef = useRef(onCameraMoved);
  const reportRef = useRef(onReport);

  cameraRef.current = onCameraMoved;
  reportRef.current = onReport;

  const handleCameraMoved = useCallback((point: ILevelPoint) => cameraRef.current(point), []);
  const handleReport = useCallback(
    (stats: ILevelStats, camera: ILevelCamera) => reportRef.current?.(stats, camera),
    []
  );

  useEffect(() => {
    if (!containerRef.current) {
      return;
    }

    const scene: LevelPreviewScene = new LevelPreviewScene({
      onCameraMoved: handleCameraMoved,
      onReport: handleReport,
    });

    sceneRef.current = scene;

    scene.mount(containerRef.current);

    return () => {
      sceneRef.current = null;
      scene.dispose();
    };
  }, [handleCameraMoved, handleReport]);

  useEffect(() => {
    sceneRef.current?.setTextures(textures);
  }, [textures, textureRevision]);

  useEffect(() => {
    sceneRef.current?.setSectors(sectors);
  }, [sectors]);

  useEffect(() => {
    sceneRef.current?.applyViewOptions(options);
  }, [options]);

  useEffect(() => {
    sceneRef.current?.setLighting(lighting);
  }, [lighting]);

  useEffect(() => {
    sceneRef.current?.setCameraOptions(camera);
  }, [camera]);

  // Taken when the extent changes, which is when a level opens rather than when a sector arrives, so streaming never
  // moves the camera out from under the person flying it.
  useEffect(() => {
    sceneRef.current?.setBounds(bounds);
  }, [bounds]);

  useEffect(() => {
    sceneRef.current?.setFrameRateLimit(settingsService.frameRateLimit);
  }, [settingsService.frameRateLimit]);

  return <div data-testid={dataTestId} id={id} className={cn(className, "h-full w-full")} ref={containerRef} />;
}
