import type { PosAdapter, PosSystemId } from "../types";
import { spotAdapter } from "./spot";
import { whiteConveyorsAdapter } from "./whiteconveyors";
import { winCleanersAdapter } from "./wincleaners";

export const adapters: Record<PosSystemId, PosAdapter> = {
  spot: spotAdapter,
  whiteconveyors: whiteConveyorsAdapter,
  wincleaners: winCleanersAdapter,
};
