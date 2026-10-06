import { Nullable } from "@xrf/types";
import { useCallback, useEffect, useState } from "react";

import { createRoots } from "@/core/assets/lib/roots";
import { assetsCommands } from "@/core/ipc/commands/assets";
import { EXrayEngineChoice, XrayEngineResolution } from "@/core/ipc/types/xrf-engine-target";

import { readEngineChoice, writeEngineChoice } from "./engine-choice-memory";

/** Which engine a game root is opened as, and what Auto would read it as. */
export interface IEngineChoice {
  /** The override remembered for the root, or Auto. */
  choice: EXrayEngineChoice;
  /** What detection found for the root, or null before it answered, with no root, or where it could not. */
  detection: Nullable<XrayEngineResolution>;
  /** Whether detection is still running for the root. */
  isDetecting: boolean;
  /** Chooses, and remembers the choice for the root. */
  setChoice: (choice: EXrayEngineChoice) => void;
}

interface IRootValue<T> {
  root: Nullable<string>;
  value: T;
}

/**
 * The engine choice of one game root: remembered per root, and detected in the backend as an open left to Auto would.
 *
 * A failed detection reports nothing rather than a problem: Auto still resolves when the root is opened.
 *
 * @param root - The game root, or null while none is picked.
 * @returns The choice, the detection and a setter.
 */
export function useEngineChoice(root: Nullable<string>): IEngineChoice {
  const [chosen, setChosen] = useState<IRootValue<EXrayEngineChoice>>(() => ({ root, value: readEngineChoice(root) }));
  const [detected, setDetected] = useState<IRootValue<Nullable<XrayEngineResolution>>>({ root: null, value: null });

  useEffect(() => {
    if (!root) {
      return;
    }

    let isCurrent: boolean = true;

    assetsCommands
      .detectEngine(createRoots([root]))
      .then((detection: Nullable<XrayEngineResolution>) => isCurrent && setDetected({ root, value: detection ?? null }))
      .catch(() => isCurrent && setDetected({ root, value: null }));

    return () => {
      isCurrent = false;
    };
  }, [root]);

  const setChoice = useCallback(
    (choice: EXrayEngineChoice): void => {
      if (root) {
        writeEngineChoice(root, choice);
      }

      setChosen({ root, value: choice });
    },
    [root]
  );

  return {
    choice: chosen.root === root ? chosen.value : readEngineChoice(root),
    detection: detected.root === root ? detected.value : null,
    isDetecting: root !== null && detected.root !== root,
    setChoice,
  };
}
