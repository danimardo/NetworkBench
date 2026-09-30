import { invokeCommand } from "./transport";
import { peerSchema, type Peer } from "../contracts/peer";
import { z } from "zod";

const pairingStartedSchema = z.object({
  pairingId: z.uuid(),
  verificationCode: z.string().regex(/^\d{6}$/, "El código debe tener seis dígitos"),
  peer: peerSchema,
});
export type PairingStarted = z.infer<typeof pairingStartedSchema>;

const pairingOutcomeSchema = z.object({
  accepted: z.boolean(),
  peer: peerSchema.nullable(),
});
export type PairingOutcome = z.infer<typeof pairingOutcomeSchema>;

/**
 * Un equipo visto por mDNS. **Ningún campo prueba identidad** (FR-011): nombre, dirección
 * y huella son lo que el equipo dice de sí. La identidad real la demuestra el certificado
 * TLS al conectar. Por eso no hay estado de confianza aquí.
 */
export const discoveredPeerSchema = z.object({
  instanceId: z.uuid(),
  fingerprintDeclarada: z.string().regex(/^[0-9a-f]{64}$/i),
  displayName: z.string(),
  addresses: z.array(z.string()),
  protocolVersion: z.number().int().nonnegative(),
  appVersion: z.string(),
  linkMbps: z.number().int().nonnegative(),
  busy: z.boolean(),
});
export type DiscoveredPeer = z.infer<typeof discoveredPeerSchema>;

export async function listDiscoveredPeers(): Promise<DiscoveredPeer[]> {
  return invokeCommand("peers_discovered_list", undefined, z.array(discoveredPeerSchema));
}

/**
 * Si un equipo guardado está activo en la red, según la comprobación periódica del backend
 * (conexión TCP al puerto de control). Es una pista para mostrar, no una prueba de
 * identidad: esa la da el certificado TLS al conectar.
 */
export const reachabilitySchema = z.object({
  fingerprint: z.string(),
  estado: z.enum(["checking", "reachable", "unreachable"]),
});
export type Reachability = z.infer<typeof reachabilitySchema>;

export async function listReachability(): Promise<Reachability[]> {
  return invokeCommand("peers_reachability_list", undefined, z.array(reachabilitySchema));
}

/** «Comprobar ahora»: un equipo concreto o, sin huella, todos los guardados. */
export async function checkPeersNow(fingerprint?: string): Promise<void> {
  await invokeCommand("peers_check_now", { fingerprint: fingerprint ?? null }, z.void().nullable());
}

/** Reinicia el descubrimiento mDNS para volver a buscar equipos en la red. */
export async function rescanPeers(): Promise<void> {
  await invokeCommand("peers_rescan", undefined, z.void().nullable());
}

export async function listPeers(): Promise<Peer[]> {
  const result = await invokeCommand("peers_list", undefined, z.array(peerSchema));
  return result;
}

export async function manualConnectPeer(host: string, port: number): Promise<Peer> {
  const result = await invokeCommand("peers_manual_connect", { host, port }, peerSchema);
  return result;
}

/**
 * Emparejamiento, primer paso: conecta y devuelve el código que hay que mostrar.
 *
 * El código NO es un secreto ni una contraseña: los dos equipos lo derivan por su
 * cuenta de sus huellas. Lo que establece la confianza es que una persona compruebe
 * que las dos pantallas muestran lo mismo (FR-012).
 *
 * No concede confianza por sí solo. El canal queda abierto hasta `confirmPairing`.
 */
export async function startPairing(host: string, port: number): Promise<PairingStarted> {
  return invokeCommand("peers_pairing_start", { host, port }, pairingStartedSchema);
}

/**
 * Segundo paso: traslada la decisión de la persona.
 *
 * Solo se guarda la confianza si este usuario acepta Y el del otro extremo también.
 */
export async function confirmPairing(
  pairingId: string,
  accepted: boolean,
): Promise<PairingOutcome> {
  return invokeCommand("peers_pairing_confirm", { pairingId, accepted }, pairingOutcomeSchema);
}

/** Marca o desmarca un equipo guardado como favorito (preferencia local). */
export async function setPeerFavorite(fingerprint: string, favorite: boolean): Promise<void> {
  await invokeCommand("peers_set_favorite", { fingerprint, favorite }, z.void().nullable());
}

/** Olvida un equipo guardado (y su confianza) solo en este equipo. */
export async function forgetPeer(fingerprint: string): Promise<void> {
  await invokeCommand("peers_forget", { fingerprint }, z.void().nullable());
}

export async function setPeerTrust(
  fingerprint: string,
  isTrusted: boolean,
  autoAccept: boolean,
): Promise<void> {
  await invokeCommand(
    "peers_set_trust",
    { fingerprint, isTrusted, autoAccept },
    z.void().nullable(),
  );
}
