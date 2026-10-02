/** One call's answer in a batch's answer, as `IRendererFetchBatch` frames it. */
export interface IFetchPart {
  /** The call's index in the batch. */
  index: number;
  status: number;
  type: string;
  bytes: ArrayBuffer;
}
