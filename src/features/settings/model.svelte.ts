import type {
  Preferences,
  DataInfo,
  AboutInfo,
  DiagnosticPaths,
  UpdateStatus,
} from "../../lib/contracts/settings";
import {
  getSettings,
  updateSettings,
  getAutostart,
  setAutostart,
  getDataInfo,
  purgeData,
  getAboutInfo,
  getDiagnosticPaths,
  openLogFolder,
  testOpenObserve,
  type OpenObserveTestInput,
} from "../../lib/api/settings";
import { checkUpdate } from "../../lib/api/updater";
import { IpcError } from "../../lib/api/transport";
import { t } from "../../lib/i18n";

/** Texto para la persona: el mensaje traducido del error de la app, no `[CÓDIGO] clave`. */
function describirError(e: unknown): string {
  if (e instanceof IpcError) return t(e.appError.messageKey);
  return e instanceof Error ? e.message : String(e);
}

const SUCCESS_MS = 3000;

export class SettingsModel {
  prefs = $state<Preferences | null>(null);
  isLoading = $state(false);
  isSaving = $state(false);
  saveError = $state<string | null>(null);
  saveSuccess = $state(false);
  /** Texto del aviso de éxito; sin él, el genérico «Ajustes guardados correctamente». */
  successMessage = $state<string | null>(null);

  autostart = $state(false);
  dataInfo = $state<DataInfo | null>(null);
  aboutInfo = $state<AboutInfo | null>(null);
  diagnosticPaths = $state<DiagnosticPaths | null>(null);
  updateStatus = $state<UpdateStatus | null>(null);
  isCheckingUpdate = $state(false);

  isTestingOpenObserve = $state(false);
  /** El texto exacto que devolvió el backend (estado HTTP + cuerpo, o el error de red), no
   * la clave genérica traducida: el botón "Probar conexión" existe justo para eso. */
  openObserveTestResult = $state<{ ok: boolean; message: string } | null>(null);

  private successTimer: ReturnType<typeof setTimeout> | null = null;

  /** Avisa de que algo salió bien con un texto propio (se cierra solo, como «guardado»). */
  flashSuccess(mensaje: string | null = null) {
    this.successMessage = mensaje;
    this.saveSuccess = true;
    if (this.successTimer) clearTimeout(this.successTimer);
    this.successTimer = setTimeout(() => {
      this.saveSuccess = false;
    }, SUCCESS_MS);
  }

  /** Muestra un error como aviso flotante, traducido. */
  reportError(e: unknown) {
    this.saveError = describirError(e);
  }

  /** Cierra el aviso de error (el de éxito se cierra solo). */
  dismissError() {
    this.saveError = null;
  }

  async load() {
    this.isLoading = true;
    this.saveError = null;
    try {
      const [p, auto, data, about, rutas] = await Promise.all([
        getSettings(),
        getAutostart().catch(() => false),
        getDataInfo().catch(() => null),
        getAboutInfo().catch(() => null),
        getDiagnosticPaths().catch(() => null),
      ]);
      this.prefs = p;
      this.autostart = auto;
      this.dataInfo = data;
      this.aboutInfo = about;
      this.diagnosticPaths = rutas;
    } catch (e: unknown) {
      this.saveError = describirError(e);
    } finally {
      this.isLoading = false;
    }
  }

  async update(partial: Partial<Preferences>): Promise<boolean> {
    if (!this.prefs) return false;
    this.isSaving = true;
    this.saveError = null;
    this.saveSuccess = false;

    const candidate: Preferences = {
      ...this.prefs,
      ...partial,
    };

    try {
      const saved = await updateSettings(candidate);
      this.prefs = saved;
      // Un guardado seguido de otro no debe cortar el aviso del segundo.
      this.flashSuccess();
      return true;
    } catch (e: unknown) {
      this.saveError = describirError(e);
      return false;
    } finally {
      this.isSaving = false;
    }
  }

  async toggleAutostart(): Promise<void> {
    const target = !this.autostart;
    try {
      await setAutostart(target);
      this.autostart = target;
      if (this.prefs) {
        await this.update({ autostart: target });
      }
    } catch (e: unknown) {
      this.saveError = describirError(e);
    }
  }

  async purgeHistory(): Promise<boolean> {
    try {
      await purgeData();
      if (this.dataInfo) {
        this.dataInfo = await getDataInfo().catch(() => null);
      }
      return true;
    } catch (e: unknown) {
      this.saveError = describirError(e);
      return false;
    }
  }

  async revealLogFolder(): Promise<void> {
    try {
      await openLogFolder();
    } catch (e: unknown) {
      this.saveError = describirError(e);
    }
  }

  async testOpenObserve(input: OpenObserveTestInput): Promise<void> {
    this.isTestingOpenObserve = true;
    this.openObserveTestResult = null;
    try {
      await testOpenObserve(input);
      this.openObserveTestResult = { ok: true, message: t("settings.openObserve.testOk") };
    } catch (e: unknown) {
      const detalle =
        e instanceof IpcError ? (e.appError.diagnosticId ?? describirError(e)) : describirError(e);
      this.openObserveTestResult = { ok: false, message: detalle };
    } finally {
      this.isTestingOpenObserve = false;
    }
  }

  async checkForUpdates(): Promise<void> {
    this.isCheckingUpdate = true;
    try {
      const status = await checkUpdate();
      this.updateStatus = status;
    } catch (e: unknown) {
      this.saveError = describirError(e);
    } finally {
      this.isCheckingUpdate = false;
    }
  }
}
