import { Box } from "@mui/material";
import { ReactElement, useEffect, useRef } from "react";
import { Texture } from "three";

import { IVisualPreviewViewOptions, VisualPreviewScene } from "@/core/visuals/components/scene";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { Nullable } from "@/lib/types/general";

interface IVisualPreviewViewportProps {
  model: Nullable<IVisualModelViews>;
  options: IVisualPreviewViewOptions;
  cameraResetToken: number;
  /** How far down each submesh collapse chain to draw: 0 is full detail, 1 is coarsest. */
  detail: number;
  /** Joint to mark, already resolved to a position, or null when nothing is selected. */
  highlightedJoint?: Nullable<[number, number, number]>;
  /** Loaded textures by submesh index, applied as they arrive. */
  textures?: ReadonlyMap<number, Texture>;
}

/**
 * Mounts the imperative preview scene and disposes it on unmount.
 *
 * The scene is created per mount rather than kept in state, so react strict mode remounting rebuilds a
 * clean webgl context instead of leaking the previous one. View options are read through a ref on mount
 * so a remount restores whatever the toolbar currently shows.
 *
 * A new model replaces the geometry in place rather than recreating the scene, so opening one visual
 * after another keeps the webgl context and the renderer alive.
 */
export function VisualPreviewViewport({
  model,
  options,
  cameraResetToken,
  detail,
  highlightedJoint = null,
  textures,
}: IVisualPreviewViewportProps): ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<Nullable<VisualPreviewScene>>(null);
  const optionsRef = useRef<IVisualPreviewViewOptions>(options);
  const detailRef = useRef<number>(detail);
  const modelRef = useRef<Nullable<IVisualModelViews>>(model);

  modelRef.current = model;

  useEffect(() => {
    if (!containerRef.current) {
      return;
    }

    const scene: VisualPreviewScene = new VisualPreviewScene(modelRef.current);

    sceneRef.current = scene;

    scene.mount(containerRef.current);
    scene.applyViewOptions(optionsRef.current);
    scene.setDetailLevel(detailRef.current);

    return () => {
      sceneRef.current = null;
      scene.dispose();
    };
  }, []);

  // The scene keeps the selected level across a model change, so this does not re-apply it.
  useEffect(() => {
    sceneRef.current?.setModel(model);
    sceneRef.current?.applyViewOptions(optionsRef.current);
  }, [model]);

  useEffect(() => {
    optionsRef.current = options;
    sceneRef.current?.applyViewOptions(options);
  }, [options]);

  useEffect(() => {
    detailRef.current = detail;
    sceneRef.current?.setDetailLevel(detail);
  }, [detail]);

  // Depends on `model` as well, because a model change clears the mark and a selection that survived it must return.
  useEffect(() => {
    sceneRef.current?.setHighlightedJoint(highlightedJoint);
  }, [highlightedJoint, model]);

  /**
   * Offers every loaded texture on each change rather than only the newest one.
   */
  useEffect(() => {
    if (!textures) {
      return;
    }

    for (const [submeshIndex, texture] of textures) {
      sceneRef.current?.applyTexture(submeshIndex, texture);
    }
  }, [textures, model]);

  useEffect(() => {
    sceneRef.current?.resetCamera();
  }, [cameraResetToken]);

  return <Box ref={containerRef} sx={{ width: "100%", height: "100%", overflow: "hidden" }} />;
}
