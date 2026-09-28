/**
 * What a change queue applies its changes through.
 */
export interface ISceneChangeHandler<T> {
  /**
   * @param object - An object waiting in a change.
   * @returns Whether it draws as last put without a stall.
   */
  canApply(object: T): boolean;
  /**
   * @param object - An object whose change applies: it draws as last put from now on.
   */
  apply(object: T): void;
  /**
   * @param key - A texture whose release applies, now that nothing drawing samples it.
   */
  releaseTexture(key: string): void;
  /** Told after every settle, once whatever it applied draws. */
  settled(): void;
}
