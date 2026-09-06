import { MINIMUM_GLOSS_POWER } from "@/applications/textures-editor/lib/texture-bump-gloss";
import { TextureMakeBumpOutcome } from "@/core/bindings/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a bump generation ends.
 *
 * A gloss too dark to show a specular response is a warning rather than a failure, exactly as in the SDK: the pair is
 * written either way, because somebody who meant to author a matte surface is not making a mistake and somebody who
 * did not wants to be told.
 *
 * @param reference - Engine reference of the texture the pair belongs to.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeTextureBumpOutcome(
  reference: string,
  outcome: IJobOutcome<TextureMakeBumpOutcome>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [reference, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not generate the bump pair",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [reference, "Neither half was written: both are encoded before either is published."].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Generating the bump pair was stopped",
    };
  }

  if (result?.isGlossTooDark) {
    return {
      details: [
        reference,
        `Gloss power is ${(result.glossPower ?? 0).toFixed(3)}, below ${MINIMUM_GLOSS_POWER}.`,
        "The pair was written; the surface will show almost no specular response.",
        result.bump,
        result.companion,
      ].join("\n"),
      severity: ENotificationSeverity.WARNING,
      title: "Generated a bump pair with almost no gloss",
    };
  }

  return {
    details: [reference, result?.bump ?? "", result?.companion ?? ""].filter(Boolean).join("\n"),
    severity: ENotificationSeverity.SUCCESS,
    title: "Generated the bump pair",
  };
}
