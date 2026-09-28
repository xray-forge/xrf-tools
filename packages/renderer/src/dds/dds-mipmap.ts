/** One mip: block bytes untouched, or texels already expanded to rgba. */
export interface IDdsMipmap {
  data: Uint8Array;
  width: number;
  height: number;
}
