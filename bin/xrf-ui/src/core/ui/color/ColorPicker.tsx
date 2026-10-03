import { Box } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useRef, useState } from "react";
import { RgbColor, RgbColorPicker } from "react-colorful";

import { RADIUS } from "@/core/theme/tokens";
import { IPickedColor } from "@/core/ui/color/color-intensity";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { DEFAULT_DRAFT_INTERVAL } from "@/lib/react/use-throttled-draft";

interface IColorPickerProps extends BaseComponentProps {
  value: IPickedColor;
  /** Told the colour while it is dragged at most once an interval, and at once when it is let go. */
  onChange: (value: IPickedColor) => void;
}

/**
 * A colour's hue and shade picked by pointer: a saturation square over a hue bar, drawn in the application's style.
 */
export function ColorPicker({
  "data-testid": dataTestId = "color-picker",
  id,
  className,
  value,
  onChange,
}: IColorPickerProps): ReactElement {
  // The colour shown while it is dragged, ahead of what the owner was told.
  const [draft, setDraft] = useState<Nullable<IPickedColor>>(null);
  const onChangeRef = useRef<(value: IPickedColor) => void>(onChange);
  const pending = useRef<Nullable<IPickedColor>>(null);
  const sentAt = useRef<number>(-Infinity);
  const timer = useRef<Nullable<ReturnType<typeof setTimeout>>>(null);

  onChangeRef.current = onChange;

  const flush = useCallback((): void => {
    timer.current = null;

    const next: Nullable<IPickedColor> = pending.current;

    pending.current = null;

    if (next) {
      sentAt.current = performance.now();
      onChangeRef.current(next);
    }
  }, []);

  const change = useCallback(
    (color: RgbColor): void => {
      setDraft(color);
      pending.current = color;

      if (timer.current !== null) {
        return;
      }

      const wait: number = sentAt.current + DEFAULT_DRAFT_INTERVAL - performance.now();

      if (wait <= 0) {
        flush();
      } else {
        timer.current = setTimeout(flush, wait);
      }
    },
    [flush]
  );

  // Letting go, of the pointer or a key, tells the last colour at once and shows the owner's again.
  const end = useCallback((): void => {
    if (timer.current !== null) {
      clearTimeout(timer.current);
    }

    flush();
    setDraft(null);
  }, [flush]);

  // A picker taken down mid-drag still tells what it was dragged to.
  useEffect(
    () => () => {
      if (timer.current !== null) {
        clearTimeout(timer.current);
        flush();
      }
    },
    [flush]
  );

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={(theme) => ({
        "& .react-colorful": { gap: 1, height: 160, width: "100%" },
        "& .react-colorful__saturation": {
          border: `1px solid ${theme.palette.divider}`,
          borderBottom: `1px solid ${theme.palette.divider}`,
          borderRadius: `${RADIUS.md}px`,
        },
        "& .react-colorful__hue": {
          border: `1px solid ${theme.palette.divider}`,
          borderRadius: `${RADIUS.md}px`,
          flex: "0 0 10px",
        },
        "& .react-colorful__last-control": { borderRadius: `${RADIUS.md}px` },
        "& .react-colorful__pointer": {
          border: `2px solid ${theme.palette.common.white}`,
          boxShadow: "0 0 0 1px rgba(0, 0, 0, 0.6)",
          height: 14,
          width: 14,
        },
      })}
      onPointerUp={end}
      onPointerCancel={end}
      onKeyUp={end}
    >
      <RgbColorPicker color={draft ?? value} onChange={change} />
    </Box>
  );
}
