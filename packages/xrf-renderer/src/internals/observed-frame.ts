/** What an observer reads of the frame: which render call it is, and the camera its uniforms are read from. */
export interface IObservedFrame {
  renderId: number;
  camera?: unknown;
}
