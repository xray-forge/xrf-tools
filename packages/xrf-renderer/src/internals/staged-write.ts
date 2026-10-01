/** Where a staged write's bytes stand for its copy: a staging buffer and the offset in it. */
export interface IStagedWrite {
  buffer: object;
  offset: number;
}
