import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useRef } from "react";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelSectorSource } from "@/core/level/lib/render/level-render-protocol";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { LevelPreviewScene } from "@/core/level/lib/scene";
import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { ILevelTextureSource } from "@/core/level/lib/texture/level-texture-set";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { SettingsService } from "@/core/settings/services/settings";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface ILevelPreviewViewportProps extends BaseComponentProps {
  /** The level's sectors, which deliver themselves as bytes for the scene to build from. */
  sectors: Nullable<ILevelSectorSource>;
  /** The level's shader table, which every arriving sector joins its surfaces against. */
  surfaces?: ReadonlyArray<XraySurfaceDescriptor>;
  /** The level's extent: what the grid is sized against, and where the camera opens. */
  bounds: Nullable<VisualBounds>;
  /**
   * Where surfaces take their textures from, owned by the loader rather than by the scene. One identity for the
   * life of a level: what changes in it, the set says for itself, to whatever subscribed.
   */
  textures?: Nullable<ILevelTextureSource>;
  options?: ILevelViewOptions;
  /** What the camera sees and how it answers input. */
  camera?: ILevelCameraOptions;
  /** What the viewer is lighting with, which is the viewer's answer rather than anything the level carries. */
  lighting?: ILevelLighting;
  /** Where the camera has gone, for the loader to stream against. */
  onCameraMoved: (point: ILevelPoint) => void;
  /** What the viewport costs and where its camera is, a few times a second while a level is open. */
  onReport?: (stats: ILevelStats, camera: ILevelCamera) => void;
  /** Takes how to ask the scene what it draws, for as long as there is a scene to ask. */
  onMeasurable?: (measure: Nullable<() => Promise<ReadonlyMap<number, ILevelSurfaceGeometry>>>) => void;
}

/**
 * Mounts the imperative level scene and disposes it on unmount.
 */
export function LevelPreviewViewport({
  "data-testid": dataTestId = "level-preview-viewport",
  id,
  className,
  sectors,
  surfaces,
  bounds,
  textures = null,
  options = DEFAULT_LEVEL_VIEW_OPTIONS,
  camera = DEFAULT_LEVEL_CAMERA_OPTIONS,
  lighting = DEFAULT_LEVEL_LIGHTING,
  onCameraMoved,
  onReport,
  onMeasurable,
}: ILevelPreviewViewportProps): ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);
  const settingsService: SettingsService = useInjection(SettingsService);
  const sceneRef = useRef<Nullable<LevelPreviewScene>>(null);

  // Held in refs so the scene is built once: rebuilding it because a handler identity changed would drop the webgl
  // context and everything uploaded into it.
  const cameraRef = useRef(onCameraMoved);
  const reportRef = useRef(onReport);
  const measurableRef = useRef(onMeasurable);

  cameraRef.current = onCameraMoved;
  reportRef.current = onReport;
  measurableRef.current = onMeasurable;

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
    measurableRef.current?.(() => Promise.resolve(scene.measureSurfaceGeometry()));

    return () => {
      sceneRef.current = null;
      measurableRef.current?.(null);
      scene.dispose();
    };
  }, [handleCameraMoved, handleReport]);

  useEffect(() => {
    sceneRef.current?.setTextures(textures);
  }, [textures]);

  // The table before the sectors: a sector arriving with no table to join against would draw untextured.
  useEffect(() => {
    sceneRef.current?.setSurfaces(surfaces ?? []);
  }, [surfaces]);

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
