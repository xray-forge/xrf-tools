/**
 * What the culls read of the static draws: how far each pool is used, one version bumped by any change a cull has to
 * see, and the uploads to make before one runs.
 */
export interface IStaticPools {
  /** Bumped whenever a slot, cluster, place, row, impostor, batch or region changes, or a buffer is replaced. */
  readonly version: number;
  readonly clusterExtent: number;
  readonly rowExtent: number;
  readonly batchExtent: number;
  readonly lodExtent: number;
  /** Entries the camera's first view can leave for its second at most: up to the end of the last surface region. */
  readonly candidateExtent: number;
  /** Queues what changed since the last upload to go up with the buffers' next use, and rebinds what grew. */
  flush(): void;
}
