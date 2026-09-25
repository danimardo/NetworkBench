import { z } from "zod";

const envSchema = z.object({
  MODE: z.string().default("development"),
  DEV: z.boolean().default(false),
  PROD: z.boolean().default(true),
  VITE_LOG_LEVEL: z.enum(["trace", "debug", "info", "warn", "error"]).optional(),
});

function loadConfig() {
  const raw = {
    MODE: import.meta.env.MODE,
    DEV: import.meta.env.DEV,
    PROD: import.meta.env.PROD,
    VITE_LOG_LEVEL: import.meta.env.VITE_LOG_LEVEL,
  };

  const parsed = envSchema.safeParse(raw);
  if (!parsed.success) {
    throw new Error(`Configuración de entorno frontend inválida: ${parsed.error.message}`);
  }

  return parsed.data;
}

export const clientConfig = loadConfig();
export type ClientConfig = typeof clientConfig;
