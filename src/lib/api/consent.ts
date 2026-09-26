import { z } from "zod";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invokeCommand } from "./transport";
import { peerSchema } from "../contracts/peer";
import { benchmarkPlanSchema } from "../contracts/plan";

/** Solicitud de sesión entrante de un equipo de confianza sin autoaceptación (T177, FR-016). */
export const incomingSessionRequestSchema = z.object({
  requestId: z.uuid(),
  peer: peerSchema,
  plan: benchmarkPlanSchema,
});
export type IncomingSessionRequest = z.infer<typeof incomingSessionRequestSchema>;

/**
 * Emparejamiento entrante (T182, FR-012). El código de seis dígitos no es un secreto: lo
 * que da confianza es que una persona compruebe que coincide con el que ve el otro equipo.
 */
export const incomingPairingRequestSchema = z.object({
  pairingId: z.uuid(),
  verificationCode: z.string().regex(/^\d{6}$/),
  peer: peerSchema,
});
export type IncomingPairingRequest = z.infer<typeof incomingPairingRequestSchema>;

export async function listIncomingSessionRequests(): Promise<IncomingSessionRequest[]> {
  return invokeCommand("session_incoming_list", undefined, z.array(incomingSessionRequestSchema));
}

export async function respondIncomingSession(requestId: string, accepted: boolean): Promise<void> {
  await invokeCommand("session_incoming_respond", { requestId, accepted }, z.void().nullable());
}

export async function listIncomingPairingRequests(): Promise<IncomingPairingRequest[]> {
  return invokeCommand(
    "peers_pairing_incoming_list",
    undefined,
    z.array(incomingPairingRequestSchema),
  );
}

export async function respondIncomingPairing(pairingId: string, accepted: boolean): Promise<void> {
  await invokeCommand(
    "peers_pairing_incoming_respond",
    { pairingId, accepted },
    z.void().nullable(),
  );
}

/** Avisa de una solicitud de sesión entrante nueva (T177, `session://incoming-request`). */
export async function onIncomingSessionRequest(callback: () => void): Promise<UnlistenFn> {
  return listen("session://incoming-request", () => callback());
}

/** Avisa de un emparejamiento entrante nuevo (T182, `peers://incoming-pairing`). */
export async function onIncomingPairingRequest(callback: () => void): Promise<UnlistenFn> {
  return listen("peers://incoming-pairing", () => callback());
}
