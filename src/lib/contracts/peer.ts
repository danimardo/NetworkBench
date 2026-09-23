import { z } from "zod";
import { instanceIdSchema } from "./common";

/**
 * Estados de confianza de un peer:
 * - unknown: descubierto o conectado por primera vez, sin verificación de huella/código.
 * - known: huella verificada previamente pero sin confianza permanente.
 * - trusted: marcado explícitamente como confiable por el usuario.
 * - trustedAutoAccept: confiable y con autoaceptación habilitada (requiere consentimiento explícito).
 */
export const trustStateSchema = z.enum(["unknown", "known", "trusted", "trustedAutoAccept"]);
export type TrustState = z.infer<typeof trustStateSchema>;

export const sha256FingerprintSchema = z
  .string()
  .regex(/^[0-9a-fA-F]{64}$/, "La huella debe ser un hash SHA-256 de 64 caracteres hexadecimales")
  .transform((val) => val.toLowerCase());

export const peerDisplayNameSchema = z
  .string()
  .min(1, "El nombre de equipo no puede estar vacío")
  .max(48, "El nombre de equipo no puede exceder 48 caracteres")
  .refine(
    (name) => !/[\u0000-\u001F\u007F-\u009F\u202A-\u202E\u2066-\u2069]/.test(name),
    "El nombre no puede contener caracteres de control ni de control bidireccional",
  );

export const peerSchema = z.object({
  instanceId: instanceIdSchema,
  displayName: peerDisplayNameSchema,
  fingerprint: sha256FingerprintSchema,
  addresses: z.array(z.string().min(1)),
  trustState: trustStateSchema,
  autoAccept: z.boolean(),
  lastSeen: z.string().datetime({ message: "lastSeen debe ser fecha UTC ISO 8601" }),
  alias: z.string().max(48).nullable().optional(),
});

export type Peer = z.infer<typeof peerSchema>;
