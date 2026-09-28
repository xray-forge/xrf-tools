/** Controls that say when a drag starts and ends, as three's `OrbitControls` do. */
export interface IDragControls {
  addEventListener(type: "start" | "end", listener: () => void): void;
  removeEventListener(type: "start" | "end", listener: () => void): void;
}
