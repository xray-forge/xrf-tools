/**
 * How a request may go together with others: requests naming one `url` and carrying the same headers are posted there
 * at once, as a JSON array of their `call`s. The server answers the calls' parts, in the order they finish: each a header
 * of the call's index in the array (`u32`), its status (`u16`), its media type's length (`u16`) and its bytes' length
 * (`u32`), little-endian, then the media type, then the bytes, which answer the call as its request alone would have.
 */
export interface IRendererFetchBatch {
  url: string;
  /** The request as one call of the batch, its JSON. */
  call: string;
}
