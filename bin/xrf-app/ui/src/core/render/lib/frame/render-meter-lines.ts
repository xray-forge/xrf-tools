import { ABSENT_VALUE, formatNumber } from "@xrf/format";
import { RenderMeterReading } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

/**
 * @param value - A luminance, or null where none was read.
 * @returns It to three figures, or whole once it reaches the thousands, as cd/m² of daylight do.
 */
export function formatRenderLuminance(value: Nullable<number>): string {
  if (value === null || !Number.isFinite(value)) {
    return ABSENT_VALUE;
  }

  return value >= 1000 ? Math.round(value).toString() : value.toPrecision(3);
}

/**
 * @param stops - Stops about middle grey, or null where none were read.
 * @returns Them signed, to hundredths.
 */
function formatStops(stops: Nullable<number>): string {
  return stops !== null && Number.isFinite(stops) && stops >= 0 ? `+${stops.toFixed(2)}` : formatNumber(stops, 2);
}

/**
 * What the spot meter read: the light in stops about middle grey as exposed, then the light before the exposure, in
 * cd/m² with the exposure value at ISO 100 where the light is physical, or in the engine's units with the exposure's
 * scale; then, where a surface lies under the spot, its albedo, gloss and distance, and the light the sun and the lamps
 * gave it. A value the meter did not read shows as absent.
 *
 * @param meter - The meter's reading.
 * @returns Its lines.
 */
export function toRenderMeterLines(meter: RenderMeterReading): Array<string> {
  const luminance: string = formatRenderLuminance(meter.luminance);
  const light: string =
    meter.ev100 === null
      ? `L ${luminance} · ×${formatNumber(meter.exposure, 2)}`
      : `L ${luminance} cd/m² · EV100 ${formatNumber(meter.ev100, 1)}`;
  const lines: Array<string> = [`Meter ${formatStops(meter.stops)} EV · ${light}`];

  if (meter.albedo !== null && meter.gloss !== null && meter.distance !== null) {
    lines.push(
      `albedo ${formatNumber(meter.albedo, 3)} · gloss ${formatNumber(meter.gloss, 2)} · ${formatNumber(meter.distance, 1)} m`
    );
  }

  if (meter.diffuse !== null && meter.specular !== null) {
    lines.push(`light ${meter.diffuse.toPrecision(3)} · specular ${meter.specular.toPrecision(3)}`);
  }

  return lines;
}
