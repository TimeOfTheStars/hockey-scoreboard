import type { GameState } from "../../../../packages/shared/types/gameState";
import { defaultGameState } from "../../../../packages/shared/types/gameState";

export function parseExternalStatePayload(raw: unknown): GameState | null {
  const one = Array.isArray(raw) ? raw[0] : raw;
  if (!one || typeof one !== "object") {
    return null;
  }

  return { ...defaultGameState, ...(one as Partial<GameState>) } as GameState;
}

