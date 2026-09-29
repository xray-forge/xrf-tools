/**
 * The engine's epsilons (`xrCore/math_constants.h`), the limits it compares floats against, under the names it gives
 * them so a port reads against its source.
 */

/** `EPS_S`, the smallest: what `fis_zero` compares against by default. */
export const EPS_S: number = 0.0000001;

/** `EPS`. */
export const EPS: number = 0.00001;

/** `EPS_L`, the largest. */
export const EPS_L: number = 0.001;
