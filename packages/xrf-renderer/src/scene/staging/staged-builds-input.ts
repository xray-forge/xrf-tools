/** What a scene's staged builds are told to do with a build. */
export interface IStagedBuildsInput<TBuild> {
  /** Takes a build down: one drawing that another replaced, one never drawn, or one let go once its compile ended. */
  release: (build: TBuild) => void;
  /** A build compiled and draws from now on. */
  onCommit?: (build: TBuild) => void;
}
