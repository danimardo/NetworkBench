import {
  createMissingFirewallRules,
  getFirewallRulesStatus,
  openNetworkSettings,
  publicNetworks,
  type FirewallRulesReport,
} from "../../lib/api/firewall";
import { getSettings, updateSettings } from "../../lib/api/settings";
import { IpcError } from "../../lib/api/transport";
import { logger } from "../../lib/logging";
import { t } from "../../lib/i18n";

/** Texto para la persona: el mensaje traducido del error de la app. */
function describirError(err: unknown): string {
  return err instanceof IpcError ? t(err.appError.messageKey) : String(err);
}

/** Por qué se avisa al arrancar. */
export type MotivoDelAviso = "faltan" | "redPublica";

/**
 * Comprobación de las reglas de firewall al arrancar la aplicación.
 *
 * Un usuario nuevo arranca sin reglas y se enteraría cuando otro equipo no le conecta. Aquí se
 * lee el estado en segundo plano (sin elevación, unos segundos) y, si algo falta, se deja un
 * aviso con una acción. **Nunca eleva por su cuenta**: el UAC solo sale al pulsar «Crear
 * ahora». Si la lectura falla no se afirma nada: no hay aviso.
 */
export class FirewallStartupCheck {
  report = $state<FirewallRulesReport | null>(null);
  /** «Ahora no»: se oculta hasta el próximo arranque. */
  descartado = $state(false);
  creando = $state(false);
  /** Texto del último error al crear (p. ej. UAC rechazado), ya traducido. */
  error = $state<string | null>(null);
  /** Texto del último aviso de éxito; se cierra solo, como en Ajustes → Cortafuegos. Sin
   * esto, crear o permitir desde el propio aviso no daba ninguna confirmación positiva —
   * solo que el aviso desaparecía, que no es lo mismo que "esto ha funcionado". */
  exito = $state<string | null>(null);
  #timerExito: ReturnType<typeof setTimeout> | null = null;
  static readonly #DURACION_EXITO_MS = 3000;

  #flashExito(mensaje: string): void {
    this.exito = mensaje;
    if (this.#timerExito) clearTimeout(this.#timerExito);
    this.#timerExito = setTimeout(() => {
      this.exito = null;
    }, FirewallStartupCheck.#DURACION_EXITO_MS);
  }

  /** Reglas que la aplicación puede crear ahora: no valen y su programa existe. */
  porCrear = $derived(
    this.report ? this.report.reglas.filter((r) => r.estado !== "present" && r.programaExiste) : [],
  );

  /** Redes públicas que las reglas actuales no cubren (el permiso está sin dar). */
  redesPublicas = $derived(
    this.report && !this.report.permitirPublico ? publicNetworks(this.report) : [],
  );

  /** Sin el ayudante elevado no se puede crear desde la app: se remite a Ajustes. */
  puedeCrear = $derived(this.report?.ayudanteDisponible ?? false);

  motivo = $derived.by<MotivoDelAviso | null>(() => {
    if (this.porCrear.length > 0) return "faltan";
    if (this.redesPublicas.length > 0) return "redPublica";
    return null;
  });

  visible = $derived(!this.descartado && this.motivo !== null);

  async comprobar(): Promise<void> {
    try {
      this.report = await getFirewallRulesStatus();
    } catch (err) {
      logger.warn({
        module: "firewall",
        eventCode: "STARTUP_CHECK_FAILED",
        message: `No se pudo comprobar el cortafuegos al arrancar: ${String(err)}`,
      });
    }
  }

  async crearAhora(): Promise<void> {
    this.creando = true;
    this.error = null;
    try {
      this.report = await createMissingFirewallRules();
      this.#flashExito(t("settings.fw.created"));
    } catch (err) {
      this.error = describirError(err);
    } finally {
      this.creando = false;
    }
  }

  /**
   * «Permitir en redes públicas» (ya confirmado por la persona): guarda el permiso, relee y
   * recrea las reglas con el perfil Público (UAC). Si el UAC se rechaza, el permiso queda
   * guardado y las reglas aparecen como desactualizadas: el aviso pasa a ofrecer «Crear ahora».
   */
  async permitirPublico(): Promise<void> {
    this.creando = true;
    this.error = null;
    try {
      const preferencias = await getSettings();
      await updateSettings({ ...preferencias, firewallAllowPublic: true });
      this.report = await getFirewallRulesStatus();
      this.report = await createMissingFirewallRules();
      this.#flashExito(t("settings.fw.created"));
    } catch (err) {
      this.error = describirError(err);
    } finally {
      this.creando = false;
    }
  }

  /** La salida limpia para una red de confianza: cambiar su tipo a «Privada» en Windows. */
  async abrirConfiguracionDeRed(): Promise<void> {
    try {
      await openNetworkSettings();
    } catch (err) {
      this.error = describirError(err);
    }
  }

  descartar(): void {
    this.descartado = true;
  }
}
