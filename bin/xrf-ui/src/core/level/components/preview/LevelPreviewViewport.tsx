import { ReactElement, useCallback, useEffect, useRef } from "react";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { LevelPreviewScene } from "@/core/level/components/scene/LevelPreviewScene";
import { ILevelPoint } from "@/core/level/lib/level-residency";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ILevelStats } from "@/core/level/lib/level-stats";
import { ILevelTextureLookup } from "@/core/level/lib/level-texture-set";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/level-view-options";
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
  options?: ILevelViewOptions;
  /** Where the camera has gone, for the loader to stream against. */
  onCameraMoved: (point: ILevelPoint) => void;
  /** Where the camera is, in the level's own coordinates, for a person flying it to read. */
  onCameraChanged?: (point: ILevelPoint) => void;
  onStats?: (stats: ILevelStats) => void;
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
  options = DEFAULT_LEVEL_VIEW_OPTIONS,
  onCameraMoved,
  onCameraChanged,
  onStats,
}: ILevelPreviewViewportProps): ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<Nullable<LevelPreviewScene>>(null);

  // Held in refs so the scene is built once: rebuilding it because a handler identity changed would drop the webgl
  // context and everything uploaded into it.
  const cameraRef = useRef(onCameraMoved);
  const readoutRef = useRef(onCameraChanged);
  const statsRef = useRef(onStats);

  cameraRef.current = onCameraMoved;
  readoutRef.current = onCameraChanged;
  statsRef.current = onStats;

  const handleCameraMoved = useCallback((point: ILevelPoint) => cameraRef.current(point), []);
  const handleCameraChanged = useCallback((point: ILevelPoint) => readoutRef.current?.(point), []);
  const handleStats = useCallback((stats: ILevelStats) => statsRef.current?.(stats), []);

  useEffect(() => {
    if (!containerRef.current) {
      return;
    }

    const scene: LevelPreviewScene = new LevelPreviewScene({
      onCameraChanged: handleCameraChanged,
      onCameraMoved: handleCameraMoved,
      onStats: handleStats,
    });

    sceneRef.current = scene;

    scene.mount(containerRef.current);

    return () => {
      sceneRef.current = null;
      scene.dispose();
    };
  }, [handleCameraChanged, handleCameraMoved, handleStats]);

  useEffect(() => {
    sceneRef.current?.setTextures(textures);
  }, [textures]);

  useEffect(() => {
    sceneRef.current?.setSectors(sectors);
  }, [sectors]);

  useEffect(() => {
    sceneRef.current?.applyViewOptions(options);
  }, [options]);

  // Taken when the extent changes, which is when a level opens rather than when a sector arrives, so streaming never
  // moves the camera out from under the person flying it.
  useEffect(() => {
    sceneRef.current?.setBounds(bounds);
  }, [bounds]);

  return <div data-testid={dataTestId} id={id} className={cn(className, "h-full w-full")} ref={containerRef} />;
}
