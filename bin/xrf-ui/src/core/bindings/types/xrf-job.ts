// Auto-generated rust bindings. Do not edit it manually.

/**
 * One snapshot of a running job, as it crosses to whoever is watching.
 *
 * The whole active stack rather than the deepest level, because a reader showing two bars needs both at the same
 * instant. Two snapshots taken a moment apart would let the outer bar describe a phase the inner one has left.
 *
 * It carries no job identity and no wall-clock timestamp. Identity belongs to whoever addressed the job — repeating
 * it in every update would make the payload the second place it can be wrong — and elapsed time is measured from a
 * monotonic start, so a clock adjustment mid-run cannot make a job appear to run backwards.
 */
export type JobProgress = {
  /** The active stack, outermost first. Never empty while a job is reporting. */
  levels: Array<ProgressLevel>;
  /** How long the job has been running, preparation included. */
  duration: number;
  /**
   * What the job is on right now, where saying so is meaningful.
   *
   * Replaced by the next snapshot and never accumulated: this is a line on screen, not a log. An operation running
   * its units across a pool leaves it empty, because naming one arbitrary worker's entry reads as thrashing rather
   * than as progress.
   */
  detail: string | null;
};

/**
 * One level of an active job's progress, as reported.
 *
 * A level counts whatever it is made of: a parent counts its finished children, a leaf counts its own units. That is
 * what lets a run reporting `["verify" 2/7, "textures" 400/40000]` and one reporting
 * `["unpack" 1/2, "write" 45000/100000]` use one mechanism instead of a phase concept and a unit concept.
 */
export type ProgressLevel = {
  /**
   * Stable machine identity, declared as a constant beside the operation that enters it.
   *
   * Separate from `label` because a reader keying on a phase must not be keying on wording somebody will improve.
   */
  id: string;
  /** What to call this level in front of a person, where the id is not already presentable. */
  label: string | null;
  completed: number;
  /**
   * Absent where the work cannot be counted before it is done.
   *
   * Honest rather than convenient: a reader shows an indeterminate state and the active phase, which is true, instead
   * of a percentage derived from a total nobody knows.
   */
  total: number | null;
  unit: ProgressUnit;
};

/**
 * What a level's counts are counting.
 *
 * Carried rather than inferred because entry counts mislead where entry sizes do not agree: an archive whose last two
 * hundred entries are level meshes sits at ninety-nine per cent for a third of its run. A reader that cannot tell a
 * count of things from a count of bytes has to hardcode per-operation knowledge to render either one.
 */
export type ProgressUnit =
  /** Discrete things: files, entries, checks, volumes. */
  | "items"
  /** Bytes, rendered through the reader's own size formatting. */
  | "bytes";
