import { z } from "zod";

export const ThemeModeSchema = z.enum(["system", "light", "dark"]);
export type ThemeMode = z.infer<typeof ThemeModeSchema>;

export const PreferencesSchema = z.object({
  schemaVersion: z.number().int().positive(),
  theme: ThemeModeSchema,
  locale: z.string().min(2).max(10),
  reduceMotion: z.boolean(),
  logLevel: z.enum(["trace", "debug", "info", "warn", "error"]),
  autoAcceptTrusted: z.boolean(),
  customControlPort: z.number().int().min(1024).max(65000).nullable().optional(),
  windowGeometry: z
    .object({
      x: z.number().int(),
      y: z.number().int(),
      width: z.number().int().positive(),
      height: z.number().int().positive(),
      isMaximized: z.boolean(),
    })
    .optional(),
  autostart: z.boolean().default(false),
  closeAction: z.enum(["ask", "minimize", "exit"]).default("ask"),
  mdnsEnabled: z.boolean().default(true),
  firewallAllowPublic: z.boolean().default(false),
  /** Envío opt-in de diagnóstico a un servidor OpenObserve propio (constitución, enmienda
   * 0.8.0 al principio IV). Apagado y vacío por defecto: enviar algo fuera de esta máquina
   * es siempre una decisión explícita de la persona. */
  openObserveEnabled: z.boolean().default(false),
  openObserveUrl: z.string().default(""),
  openObserveOrg: z.string().default(""),
  openObserveStream: z.string().default(""),
  openObserveToken: z.string().default(""),
});

export type Preferences = z.infer<typeof PreferencesSchema>;

export const DataInfoSchema = z.object({
  dbPath: z.string(),
  dbSizeBytes: z.number().int().nonnegative(),
  settingsPath: z.string(),
  settingsSizeBytes: z.number().int().nonnegative(),
});

export type DataInfo = z.infer<typeof DataInfoSchema>;

export const AboutInfoSchema = z.object({
  appName: z.string(),
  appVersion: z.string(),
  engineName: z.string(),
  engineVersion: z.string(),
  protocolVersion: z.string(),
  license: z.string(),
  copyright: z.string(),
  buildHash: z.string(),
  buildDate: z.string(),
  buildDirty: z.boolean(),
});

export type AboutInfo = z.infer<typeof AboutInfoSchema>;

export const DiagnosticPathsSchema = z.object({
  appLogDir: z.string(),
  firewallHelperLogPath: z.string(),
});

export type DiagnosticPaths = z.infer<typeof DiagnosticPathsSchema>;

export const UpdateStatusSchema = z.discriminatedUnion("status", [
  z.object({ status: z.literal("upToDate") }),
  z.object({
    status: z.literal("updateAvailable"),
    version: z.string(),
    notes: z.string(),
    downloadUrl: z.string(),
    signature: z.string(),
  }),
  z.object({ status: z.literal("deferredDueToActiveSession") }),
]);

export type UpdateStatus = z.infer<typeof UpdateStatusSchema>;
