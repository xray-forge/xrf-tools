import {
  AmbientLight,
  AxesHelper,
  BufferGeometry,
  Color,
  DataTexture,
  DirectionalLight,
  GridHelper,
  Mesh,
  MeshStandardMaterial,
  PerspectiveCamera,
  Scene,
  Texture,
  WebGLRenderer,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import {
  DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
  IVisualPreviewSceneConfig,
} from "@/core/visuals/components/scene/scene-config";
import { createCheckerTexture, createSubmeshGeometry } from "@/core/visuals/components/scene/VisualPreviewScene.utils";
import { getVisualSubmeshLevel, IVisualModelViews, IVisualSubmeshLevel } from "@/core/visuals/lib/visual-views";
import { Nullable } from "@/lib/types/general";

/**
 * How a preview looks before anyone touches a toggle.
 */
export const DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS: IVisualPreviewViewOptions = {
  isWireframe: false,
  isGridVisible: true,
  isAxesVisible: true,
  isCheckerVisible: false,
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
}

/**
 * One drawn submesh: its mesh, its own material, and the texture applied to it.
 */
interface IVisualSubmeshMesh {
  mesh: Mesh<BufferGeometry, MeshStandardMaterial>;
  texture: Nullable<Texture>;
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

  /**
   * Meshes keyed by the submesh index they were built from.
   *
   * Keyed rather than ordered because textures arrive addressed by that index, out of order and after the fact, so a
   * position in an array would be the wrong thing to trust.
   */
  private meshes: Map<number, IVisualSubmeshMesh> = new Map();
  /** The last options applied, so a texture landing later knows whether the checker is currently covering it. */
  private viewOptions: Nullable<IVisualPreviewViewOptions> = null;
  private model: Nullable<IVisualModelViews> = null;
  /**
   * How far down its collapse chain every mesh is currently drawing, 0 being full detail.
   *
   * Held here so a model replaced while detail is reduced arrives reduced, rather than snapping back to full and
   * leaving the toolbar saying otherwise.
   */
  private detail: number = 0;
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
   * Geometry is rebuilt while the renderer and controls survive. The camera is refitted to the replacement model.
   *
   * @param model - Model views to display, or `null` to clear the scene.
   */
  public setModel(model: Nullable<IVisualModelViews>): void {
    this.clearModel();

    this.model = model;

    for (const submesh of model?.submeshes ?? []) {
      const mesh: Mesh<BufferGeometry, MeshStandardMaterial> = new Mesh(
        createSubmeshGeometry(submesh, this.detail),
        new MeshStandardMaterial({ color: this.config.meshColor, metalness: 0.05, roughness: 0.75 })
      );

      mesh.name = submesh.label;

      this.meshes.set(submesh.index, { mesh, texture: null });
      this.scene.add(mesh);
    }

    if (this.viewOptions) {
      this.applyViewOptions(this.viewOptions);
    }

    this.applyScale();
    this.resetCamera();
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

    for (const submesh of this.model?.submeshes ?? []) {
      const drawn: Nullable<IVisualSubmeshMesh> = this.meshes.get(submesh.index) ?? null;

      if (drawn) {
        const level: IVisualSubmeshLevel = getVisualSubmeshLevel(submesh, detail);

        drawn.mesh.geometry.setDrawRange(level.start, level.count);
      }
    }
  }

  /**
   * Put a loaded texture on one submesh.
   *
   * Applied per submesh rather than per model because a visual's children each declare their own reference and they
   * arrive one at a time, so a model shows its first texture without waiting for its last.
   *
   * A texture for a submesh this model does not have is disposed rather than kept: it belongs to a model the user has
   * already moved past, and holding it would leak the upload.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param texture - Uploaded texture the scene takes ownership of.
   */
  public applyTexture(submeshIndex: number, texture: Texture): void {
    const drawn: Nullable<IVisualSubmeshMesh> = this.meshes.get(submeshIndex) ?? null;

    if (!drawn) {
      texture.dispose();

      return;
    }

    // Idempotent because the caller is a react effect that re-runs whenever any texture lands, so it re-offers the ones
    // already applied. Without this the previous line would dispose the texture still in use.
    if (drawn.texture === texture) {
      return;
    }

    drawn.texture?.dispose();
    drawn.texture = texture;

    if (!this.viewOptions?.isCheckerVisible) {
      drawn.mesh.material.map = texture;
      drawn.mesh.material.needsUpdate = true;
    }
  }

  /**
   * Applies toolbar view toggles to every mesh and helper in the scene.
   *
   * @param options - Wireframe, checkerboard, grid, and axes visibility to retain for later texture arrivals.
   */
  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.viewOptions = options;

    for (const { mesh, texture } of this.meshes.values()) {
      mesh.material.wireframe = options.isWireframe;
      mesh.material.map = options.isCheckerVisible ? this.checker : texture;
      mesh.material.needsUpdate = true;
    }

    this.grid.visible = options.isGridVisible;
    this.axes.visible = options.isAxesVisible;
  }

  /**
   * Frame the model from its measured extent.
   *
   * A constant distance cannot serve this viewer: loose visuals run from a pistol a few centimetres across to an actor
   * two metres tall, so resetting the camera re-fits rather than returning to a fixed point.
   */
  public resetCamera(): void {
    const { cameraFieldOfView, cameraFitMargin, cameraDirection } = this.config;

    const radius: number = this.model?.fit.radius ?? FALLBACK_RADIUS;
    const [x, y, z] = this.model?.fit.center ?? [0, 0, 0];
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
    for (const { mesh, texture } of this.meshes.values()) {
      this.scene.remove(mesh);
      mesh.geometry.dispose();
      mesh.material.dispose();
      texture?.dispose();
    }

    this.meshes = new Map();
  }

  /** Size the helpers to the model, so the grid reads as ground rather than as a backdrop. */
  private applyScale(): void {
    const radius: number = this.model?.fit.radius ?? FALLBACK_RADIUS;

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

  private renderFrame(): void {
    this.frameHandle = requestAnimationFrame(() => this.renderFrame());

    this.applyPendingResize();

    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }
}
