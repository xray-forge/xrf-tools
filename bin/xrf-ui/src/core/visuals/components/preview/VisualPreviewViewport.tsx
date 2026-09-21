import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useRef } from "react";
import { Texture } from "three";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { SettingsService } from "@/core/settings/services/settings";
import { ViewportControls } from "@/core/ui/media/ViewportControls";
import { IVisualPreviewViewOptions, VisualPreviewScene } from "@/core/visuals/lib/scene";
import { DEFAULT_VISUAL_LIGHTING } from "@/core/visuals/lib/scene/visual-lighting";
import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { DOLLY_STEP } from "@/lib/media/orbit-dolly";
import { Nullable } from "@/lib/types/general";

/** Stable, so a viewport given no hidden bones does not re-apply an empty set on every render. */
const EMPTY_BONES: ReadonlySet<number> = new Set();

export interface IVisualPreviewViewportProps {
  model: Nullable<IVisualModelViews>;
  options: IVisualPreviewViewOptions;
  /** What the model is lit by, the viewer's own default until a surface states otherwise. */
  lighting?: IRenderLighting;
  /** How far down each submesh collapse chain to draw: 0 is full detail, 1 is coarsest. */
  detail: number;
  /** Baked bone transforms of a playing motion, or null when the model should show its bind pose. */
  motionTransforms?: Nullable<Float32Array>;
  /** Which frame of those transforms to show, and how many floats one bone occupies in them. */
  motionFrame?: number;
  motionFloatsPerBone?: number;
  /** Joint to mark, already resolved to a position, or null when nothing is selected. */
  highlightedJoint?: Nullable<[number, number, number]>;
  /** Bones to collapse, by index, already including the descendants each one hides. */
  hiddenBones?: ReadonlySet<number>;
  /** Loaded textures by submesh index, applied as they arrive. */
  textures?: ReadonlyMap<number, Texture>;
  /** Loaded bump pairs by submesh index, shaded as they arrive. */
  bumps?: ReadonlyMap<number, IVisualBumpTextures>;
}

/**
 * Mounts the imperative preview scene and disposes it on unmount.
 *
 * The scene is created per mount rather than kept in state, so react strict mode remounting rebuilds a
 * clean webgl context instead of leaking the previous one. The prop effects initialize each new scene
 * and keep it synchronized with the current model and toolbar settings.
 *
 * A new model replaces the geometry in place rather than recreating the scene, so opening one visual
 * after another keeps the webgl context and the renderer alive.
 */
export function VisualPreviewViewport({
  model,
  options,
  lighting = DEFAULT_VISUAL_LIGHTING,
  detail,
  highlightedJoint = null,
  hiddenBones,
  motionTransforms = null,
  motionFrame = 0,
  motionFloatsPerBone = 0,
  textures,
  bumps,
}: IVisualPreviewViewportProps): ReactElement {
  const containerRef = useRef<HTMLDivElement>(null);
  const settingsService: SettingsService = useInjection(SettingsService);
  const sceneRef = useRef<Nullable<VisualPreviewScene>>(null);

  useEffect(() => {
    if (!containerRef.current) {
      return;
    }

    const scene: VisualPreviewScene = new VisualPreviewScene(new DomRenderTarget(containerRef.current), null);

    sceneRef.current = scene;

    return () => {
      sceneRef.current = null;
      scene.dispose();
    };
  }, []);

  // The scene keeps view options and detail across model changes.
  useEffect(() => {
    sceneRef.current?.setModel(model);
  }, [model]);

  useEffect(() => {
    sceneRef.current?.applyViewOptions(options);
  }, [options]);

  useEffect(() => {
    sceneRef.current?.setLighting(lighting);
  }, [lighting]);

  useEffect(() => {
    sceneRef.current?.setDetailLevel(detail);
  }, [detail]);

  /**
   * Marks the selected joint, and marks it again whenever the model changes.
   */
  useEffect(() => {
    sceneRef.current?.setHighlightedJoint(highlightedJoint);
  }, [highlightedJoint, model]);

  /** Poses the mesh and the skeleton overlay, or returns both to the bind pose when nothing is playing. */
  useEffect(() => {
    sceneRef.current?.setPose(motionTransforms, motionFrame, motionFloatsPerBone);
  }, [motionTransforms, motionFrame, motionFloatsPerBone]);

  /** Collapses the hidden bones, the way the engine collapses a part that is not attached. */
  useEffect(() => {
    sceneRef.current?.setHiddenBones(hiddenBones ?? EMPTY_BONES);
  }, [hiddenBones]);

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

  /**
   * Shades every loaded bump pair on each change, the same way, since a model change rebuilds the meshes.
   */
  useEffect(() => {
    if (!bumps) {
      return;
    }

    for (const [submeshIndex, pair] of bumps) {
      sceneRef.current?.applyBump(submeshIndex, pair);
    }
  }, [bumps, model]);

  const onZoomIn = useCallback((): void => sceneRef.current?.dolly(1 / DOLLY_STEP), []);

  const onZoomOut = useCallback((): void => sceneRef.current?.dolly(DOLLY_STEP), []);

  const onReset = useCallback((): void => sceneRef.current?.resetCamera(), []);

  useEffect(() => {
    sceneRef.current?.setFrameRateLimit(settingsService.frameRateLimit);
  }, [settingsService.frameRateLimit]);

  return (
    <div className={"relative size-full"}>
      <div ref={containerRef} className={"size-full overflow-hidden"} />

      {model ? <ViewportControls onZoomIn={onZoomIn} onZoomOut={onZoomOut} onReset={onReset} /> : null}
    </div>
  );
}
