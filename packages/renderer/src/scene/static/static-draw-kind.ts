/**
 * How a static draw finds where it stands, which is the shader it is drawn with.
 */
export enum EStaticDrawKind {
  /** One place, its slot's matrix: a section of an object its matrix places. */
  SINGLE = "single",
  /** A place per instance, listed by the cull for every instance it kept: a section of an instanced object. */
  LISTED = "listed",
}
