/**
 * The level editor's open form fields by the id each is remembered under: its form reads them, and a tool handing a
 * scene over to it writes them, so a renamed field cannot leave a hand-over filling one nothing reads.
 */
export enum ELevelEditorField {
  /** The scene's `.level`. */
  SCENE = "scene",
  /** The roots the scene's library objects are looked for in, a list kept through `PATH_LIST_CODEC`. */
  OBJECTS = "objects",
  /** The game roots its textures and configs come from, a list kept through `PATH_LIST_CODEC`. */
  GAME = "game",
}
