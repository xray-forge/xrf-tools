import {
  AmbientLight,
  AxesHelper,
  BufferAttribute,
  BufferGeometry,
  Color,
  DataTexture,
  DirectionalLight,
  GridHelper,
  PerspectiveCamera,
  Points,
  PointsMaterial,
  Scene,
  Texture,
  WebGLRenderer,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import {
  DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
  IVisualPreviewSceneConfig,
} from "@/core/visuals/components/scene/scene-config";
import { VisualPreviewModel } from "@/core/visuals/components/scene/VisualPreviewModel";
import { createCheckerTexture } from "@/core/visuals/components/scene/VisualPreviewScene.utils";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { Nullable } from "@/lib/types/general";

/**
 * How a preview looks before anyone touches a toggle.
 */
export const DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS: IVisualPreviewViewOptions = {
  isWireframe: false,
  isGridVisible: true,
  isAxesVisible: true,
  isCheckerVisible: false,
  isSkeletonVisible: false,
};

/** Radius assumed when a model reports no usable extent, so the camera and helpers still have a scale. */
const FALLBACK_RADIUS: number = 1;

/**
 * View state the toolbar owns and the scene applies.
 *
 * React holds the state, the scene stays a sink: it never reports view state back, so there is one source of truth and
 * no synchronisation between react and the scene graph. Distinct from the scene's configuration, which is chosen once
 * and describes how the preview looks rather than what the user is toggling.
 */
export interface IVisualPreviewViewOptions {
  isWireframe: boolean;
  isGridVisible: boolean;
  isAxesVisible: boolean;
  /**
   * Renders a repeating checkerboard from the uv buffer instead of a flat surface.
   *
   * Present before textures are: it is the only way to see that the v flip is right, which otherwise stays invisible
   * until textures land and come out mirrored.
   */
  isCheckerVisible: boolean;
  /**
   * Draws the bind pose over the mesh.
   *
   * Rendered with depth testing off, because a skeleton is only useful when it can be seen through the mesh it sits
   * inside - which is the point when checking where a weapon's attach bone actually is.
   */
  isSkeletonVisible: boolean;
}

/**
 * Owns the three.js scene imperatively, outside of react state.
 *
 * An editor scene graph is long lived and mutated by direct manipulation, so it is deliberately not expressed as react
 * elements: react only mounts it into a container and disposes it again. Everything webgl touches stays behind this
 * class.
 */
export class VisualPreviewScene {
  private readonly config: IVisualPreviewSceneConfig;
  private readonly scene: Scene;
  private readonly camera: PerspectiveCamera;
  private readonly renderer: WebGLRenderer;
  private readonly controls: OrbitControls;
  private readonly checker: DataTexture;
  private readonly grid: GridHelper;
  private readonly axes: AxesHelper;
  private readonly resizeObserver: ResizeObserver;

  /** The model on screen, or null when nothing is open. */
  private model: Nullable<VisualPreviewModel> = null;
  /** The marker for a joint named elsewhere, kept across models rather than rebuilt. */
  private highlight: Nullable<Points<BufferGeometry, PointsMaterial>> = null;
  /** Where the marker points, kept so a toggle can show it again without the selection being sent a second time. */
  private highlightedJoint: Nullable<[number, number, number]> = null;
  /** The last options applied, so a texture landing later knows whether the checker is currently covering it. */
  private viewOptions: Nullable<IVisualPreviewViewOptions> = null;
  /** What the backend packed, kept for the extent the camera and the helpers are sized against. */
  private views: Nullable<IVisualModelViews> = null;
  /**
   * How far down its collapse chain every mesh is currently drawing, 0 being full detail.
   *
   * Held here so a model replaced while detail is reduced arrives reduced, rather than snapping back to full and
   * leaving the toolbar saying otherwise.
   */
  private detail: number = 0;
  /**
   * Whether the camera has ever been fitted to anything in this scene.
   */
  private hasFramed: boolean = false;
  /**
   * The pose and the hidden bones last asked for, kept because they outlive any one model.
   *
   * Both are stated against a skeleton rather than against one model's geometry, so a replacement wears them straight
   * away instead of flashing its bind pose with every part attached until the owner sends the same state again.
   */
  private pose: { transforms: Nullable<Float32Array>; frame: number; floatsPerBone: number } = {
    floatsPerBone: 0,
    frame: 0,
    transforms: null,
  };
  private hiddenBones: ReadonlySet<number> = new Set();
  private container: Nullable<HTMLElement> = null;
  private frameHandle: number = 0;
  private isResizePending: boolean = false;
  private renderedWidth: number = 0;
  private renderedHeight: number = 0;

  public constructor(
    model: Nullable<IVisualModelViews>,
    config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
  ) {
    this.config = config;

    this.renderer = new WebGLRenderer({ antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.domElement.style.display = "block";

    this.scene = new Scene();
    this.scene.background = new Color(config.backgroundColor);

    this.camera = new PerspectiveCamera(config.cameraFieldOfView, 1, 0.001, 10000);

    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;

    this.checker = createCheckerTexture(config);
    this.grid = new GridHelper(10, 10, config.gridColor, config.gridColor);
    this.axes = new AxesHelper(1);

    const light: DirectionalLight = new DirectionalLight(0xffffff, 2);

    light.position.set(3, 5, 4);

    this.scene.add(new AmbientLight(0xffffff, 1.4));
    this.scene.add(light);
    this.scene.add(this.grid);
    this.scene.add(this.axes);

    this.resizeObserver = new ResizeObserver(() => this.resize());

    this.setModel(model);
  }

  /**
   * Replace whatever is on screen with a different model, or with nothing.
   *
   * Geometry is rebuilt while the renderer and controls survive. The camera is fitted for the first model this scene
   * shows and held for every one after it: stepping through a tree is comparing models, and a refit per model throws
   * away the angle and the distance the comparison is being made from. Held across the empty viewport between two
   * models as well, since a load clears the screen before the replacement lands. A model of a very different size can
   * end up out of frame that way, which is what the toolbar's reset is for.
   *
   * @param views - Model views to display, or `null` to clear the scene.
   */
  public setModel(views: Nullable<IVisualModelViews>): void {
    this.clearModel();

    this.views = views;

    this.model = views
      ? VisualPreviewModel.create(views, this.scene, {
          checker: this.checker,
          detail: this.detail,
          meshColor: this.config.meshColor,
          skeletonColor: this.config.skeletonColor,
        })
      : null;

    if (this.viewOptions) {
      this.applyViewOptions(this.viewOptions);
    }

    this.applyScale();
    this.applySkeletonState();

    if (!this.hasFramed && views) {
      this.resetCamera();
    }
  }

  /**
   * Draw every mesh at a different point along its collapse chain.
   *
   * Only a draw range changes: all levels are already in the uploaded index buffer, so this touches no attribute and
   * costs no upload — which is what makes dragging the control smooth on a model carrying nine hundred levels.
   * Bounding spheres are left alone deliberately: they describe the same geometry, and refitting the camera on every
   * step would make comparing detail impossible.
   *
   * @param detail - How far down each chain to go: 0 is full detail, 1 is the coarsest each submesh has.
   */
  public setDetailLevel(detail: number): void {
    this.detail = detail;

    this.model?.setDetailLevel(detail);
  }

  /**
   * Poses the model from one frame of a baked motion, or returns it to its bind pose.
   *
   * Recorded as well as forwarded, because a model opened later has to arrive wearing it. What a frame costs and how it
   * is indexed belongs to `VisualPreviewSkeleton`.
   *
   * @param transforms - Every frame's bone transforms, frame major, or null to show the bind pose again.
   * @param frame - Which frame of that buffer to show.
   * @param floatsPerBone - Floats one bone occupies, as the bake reported it.
   */
  public setPose(transforms: Nullable<Float32Array>, frame: number, floatsPerBone: number): void {
    this.pose = { floatsPerBone, frame, transforms };

    this.model?.setPose(transforms, frame, floatsPerBone);
  }

  /**
   * Collapses some of the model's bones, the way the engine hides a part that is not attached.
   *
   * Recorded as well as forwarded, for the same reason the pose is.
   *
   * @param bones - Indices of bones to collapse, already including their descendants.
   */
  public setHiddenBones(bones: ReadonlySet<number>): void {
    this.hiddenBones = bones;

    this.model?.setHiddenBones(bones);
  }

  /**
   * Marks one joint in the viewport, or clears the mark.
   *
   * @param position - Joint position in renderer space, or null to clear the mark.
   */
  public setHighlightedJoint(position: Nullable<[number, number, number]>): void {
    this.highlightedJoint = position;

    if (position && !this.highlight) {
      const geometry: BufferGeometry = new BufferGeometry();

      geometry.setAttribute("position", new BufferAttribute(new Float32Array(3), 3));

      this.highlight = new Points(
        geometry,
        new PointsMaterial({
          color: this.config.highlightColor,
          size: this.config.highlightSize,
          sizeAttenuation: false,
          depthTest: false,
          transparent: true,
        })
      );
      this.highlight.renderOrder = 2;

      this.scene.add(this.highlight);
    }

    if (position && this.highlight) {
      const attribute: BufferAttribute = this.highlight.geometry.getAttribute("position") as BufferAttribute;

      attribute.setXYZ(0, position[0], position[1], position[2]);
      attribute.needsUpdate = true;

      // The marker is a single point, so its bounding sphere is stale after a move and frustum culling would drop it.
      this.highlight.geometry.computeBoundingSphere();
    }

    this.applyHighlightVisibility();
  }

  /**
   * Shows the joint marker only when there is one to show and the overlay it belongs to is on.
   *
   * One place decides it, because two inputs govern it - the selection and the toggle - and either can change without
   * the other.
   */
  private applyHighlightVisibility(): void {
    if (this.highlight) {
      this.highlight.visible = Boolean(this.highlightedJoint) && (this.viewOptions?.isSkeletonVisible ?? false);
    }
  }

  /**
   * Draws one of the model's submeshes with a texture, borrowing it from whoever loaded it.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param texture - Uploaded texture to draw with.
   */
  public applyTexture(submeshIndex: number, texture: Texture): void {
    this.model?.applyTexture(submeshIndex, texture);
  }

  /**
   * Applies toolbar view toggles to every mesh and helper in the scene.
   *
   * @param options - Wireframe, checkerboard, grid, and axes visibility to retain for later texture arrivals.
   */
  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.viewOptions = options;

    this.model?.applyViewOptions(options);

    this.grid.visible = options.isGridVisible;
    this.axes.visible = options.isAxesVisible;

    this.applyHighlightVisibility();
  }

  /**
   * Frame the model from its measured extent.
   *
   * A constant distance cannot serve this viewer: loose visuals run from a pistol a few centimetres across to an actor
   * two metres tall, so resetting the camera re-fits rather than returning to a fixed point.
   */
  public resetCamera(): void {
    const { cameraFieldOfView, cameraFitMargin, cameraDirection } = this.config;

    this.hasFramed = true;

    const radius: number = this.views?.fit.radius ?? FALLBACK_RADIUS;
    const [x, y, z] = this.views?.fit.center ?? [0, 0, 0];
    const distance: number = (radius / Math.sin((cameraFieldOfView * Math.PI) / 360)) * cameraFitMargin;
    const length: number = Math.hypot(cameraDirection[0], cameraDirection[1], cameraDirection[2]);

    this.camera.position.set(
      x + (cameraDirection[0] / length) * distance,
      y + (cameraDirection[1] / length) * distance,
      z + (cameraDirection[2] / length) * distance
    );
    this.camera.near = Math.max(distance / 1000, 0.0001);
    this.camera.far = distance * 100;
    this.camera.updateProjectionMatrix();

    this.controls.target.set(x, y, z);
    this.controls.update();
  }

  /**
   * Attaches the renderer to a viewport and starts its render loop.
   *
   * @param container - Element whose dimensions drive the renderer and camera aspect ratio.
   */
  public mount(container: HTMLElement): void {
    this.container = container;
    container.appendChild(this.renderer.domElement);

    this.resizeObserver.observe(container);
    this.resize();
    this.renderFrame();
  }

  /** Stops rendering, detaches the canvas, and releases the scene's WebGL resources. */
  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.resizeObserver.disconnect();
    this.controls.dispose();
    this.clearModel();

    if (this.highlight) {
      this.scene.remove(this.highlight);
      this.highlight.geometry.dispose();
      this.highlight.material.dispose();
      this.highlight = null;
    }

    this.checker.dispose();
    this.renderer.dispose();
    this.renderer.domElement.remove();

    this.container = null;
  }

  /**
   * Take the current model off the scene and free everything it owns.
   *
   * Materials and textures are per submesh now, so they are the model's to free rather than the scene's: leaving them
   * behind would leak one upload per submesh every time the user opens another visual.
   */
  private clearModel(): void {
    this.model?.dispose();
    this.model = null;
    this.views = null;

    // The marker itself survives a model change, but what it was pointing at does not. The owner re-sends the
    // selection against the replacement model, which resolves to nothing when that model has no such bone.
    this.highlightedJoint = null;
    this.applyHighlightVisibility();
  }

  /** Size the helpers to the model, so the grid reads as ground rather than as a backdrop. */
  private applyScale(): void {
    const radius: number = this.views?.fit.radius ?? FALLBACK_RADIUS;

    this.grid.scale.setScalar(radius / 2);
    this.axes.scale.setScalar(radius);
  }

  /**
   * Note a size change without acting on it.
   *
   * `setSize` clears the drawing buffer, and doing that in the observer callback can paint before the frame that
   * refills it. Recording the request and applying it immediately before the next render keeps both in one frame, and
   * needs no timer: the frame loop is already the rate limit.
   */
  private resize(): void {
    this.isResizePending = true;
  }

  private applyPendingResize(): void {
    if (!this.isResizePending || !this.container) {
      return;
    }

    const width: number = this.container.clientWidth;
    const height: number = this.container.clientHeight;

    if (!width || !height) {
      return;
    }

    this.isResizePending = false;

    if (width === this.renderedWidth && height === this.renderedHeight) {
      return;
    }

    this.renderedWidth = width;
    this.renderedHeight = height;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  /**
   * Dresses a newly built skeleton in the pose and the hidden bones the scene is holding.
   *
   * The scene holds them rather than the skeleton, because a skeleton lives and dies with one model while these outlive
   * it: both are stated against bones rather than against one model's geometry.
   */
  private applySkeletonState(): void {
    this.model?.setPose(this.pose.transforms, this.pose.frame, this.pose.floatsPerBone);
    this.model?.setHiddenBones(this.hiddenBones);
  }

  private renderFrame(): void {
    this.frameHandle = requestAnimationFrame(() => this.renderFrame());

    this.applyPendingResize();

    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }
}
