import { ComputeNode, PerspectiveCamera, StorageBufferAttribute, WebGPURenderer } from "three/webgpu";

import { createLightBinning } from "#/scene/lights/light-clusters.tsl";
import { LIGHT_VECTORS, MAX_LIGHTS } from "#/scene/lights/light-record";
import { LIGHT_CLUSTER_CAPACITY, LIGHT_CLUSTERS, LightsUniforms } from "#/uniforms/lights-uniforms";

/**
 * The clusters of the view the lights in view are binned into, a compute a frame, and what the last binning read back
 * left out: the clusters more lights reached than they hold.
 */
export class LightClusters {
  /** Lights reaching each cluster, and which they are. */
  public readonly counts: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(LIGHT_CLUSTERS), 1);
  public readonly items: StorageBufferAttribute = new StorageBufferAttribute(
    new Uint32Array(LIGHT_CLUSTERS * LIGHT_CLUSTER_CAPACITY),
    1
  );
  /** Lights each cluster was reached by and could not hold, as the binning left them. */
  public readonly drops: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(LIGHT_CLUSTERS), 1);
  public readonly uniforms: LightsUniforms = new LightsUniforms();
  /** Clusters the last read found more lights reached than they hold. */
  public fullClusters: number = 0;
  /** Lights left out of a cluster they reached, as the last read found them: a light missing from two counts twice. */
  public droppedLights: number = 0;

  private readonly binning: ComputeNode;
  /** Bumped by every forgetting, so a read begun before one lands on nothing. */
  private generation: number = 0;
  private isReading: boolean = false;

  /**
   * @param records - The lights in view, which the binning reads each one's sphere from.
   */
  public constructor(records: StorageBufferAttribute) {
    this.binning = createLightBinning(
      { counts: this.counts, drops: this.drops, items: this.items, records },
      this.uniforms,
      LIGHT_VECTORS,
      MAX_LIGHTS
    );
  }

  /**
   * @param camera - The camera drawing the frame, whose view the clusters cut.
   * @param count - Lights standing in view.
   */
  public follow(camera: PerspectiveCamera, count: number): void {
    this.uniforms.follow(camera, count);

    if (count === 0) {
      this.forget();
    }
  }

  /**
   * @param renderer - The renderer drawing.
   */
  public bin(renderer: WebGPURenderer): void {
    renderer.compute(this.binning);
  }

  /**
   * Reads back what the binning left out, for a later report; one read at a time, none while no light is in view.
   *
   * @param renderer - The renderer the clusters were binned by.
   */
  public readDrops(renderer: WebGPURenderer): void {
    if (this.isReading || this.uniforms.count.value === 0) {
      return;
    }

    const generation: number = this.generation;

    this.isReading = true;
    renderer
      .getArrayBufferAsync(this.drops)
      .then((buffer: ArrayBuffer) => {
        if (generation !== this.generation) {
          return;
        }

        const drops: Uint32Array = new Uint32Array(buffer);

        this.fullClusters = drops.reduce((total: number, it: number) => total + (it > 0 ? 1 : 0), 0);
        this.droppedLights = drops.reduce((total: number, it: number) => total + it, 0);
      })
      .catch((error: unknown) => console.error("The lights' clusters could not be read back:", error))
      .finally(() => (this.isReading = false));
  }

  /** Forgets what was read, for a view no light stands in, or lights gone. */
  public forget(): void {
    this.generation += 1;
    this.fullClusters = 0;
    this.droppedLights = 0;
  }

  public dispose(): void {
    this.forget();
    this.binning.dispose();
  }
}
