import type { Preferences, DataInfo, AboutInfo, UpdateStatus } from "../../lib/contracts/settings";
import {
  getSettings,
  updateSettings,
  getAutostart,
  setAutostart,
  getDataInfo,
  purgeData,
  getAboutInfo,
} from "../../lib/api/settings";
import { checkUpdate } from "../../lib/api/updater";

export class SettingsModel {
  prefs = $state<Preferences | null>(null);
  isLoading = $state(false);
  isSaving = $state(false);
  saveError = $state<string | null>(null);
  saveSuccess = $state(false);

  autostart = $state(false);
  dataInfo = $state<DataInfo | null>(null);
  aboutInfo = $state<AboutInfo | null>(null);
  updateStatus = $state<UpdateStatus | null>(null);
  isCheckingUpdate = $state(false);

  async load() {
    this.isLoading = true;
    this.saveError = null;
    try {
      const [p, auto, data, about] = await Promise.all([
        getSettings(),
        getAutostart().catch(() => false),
        getDataInfo().catch(() => null),
        getAboutInfo().catch(() => null),
      ]);
      this.prefs = p;
      this.autostart = auto;
      this.dataInfo = data;
      this.aboutInfo = about;
    } catch (e: unknown) {
      this.saveError = e instanceof Error ? e.message : String(e);
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
      this.saveSuccess = true;
      setTimeout(() => {
        this.saveSuccess = false;
      }, 3000);
      return true;
    } catch (e: unknown) {
      this.saveError = e instanceof Error ? e.message : String(e);
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
      this.saveError = e instanceof Error ? e.message : String(e);
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
      this.saveError = e instanceof Error ? e.message : String(e);
      return false;
    }
  }

  async checkForUpdates(): Promise<void> {
    this.isCheckingUpdate = true;
    try {
      const status = await checkUpdate();
      this.updateStatus = status;
    } catch (e: unknown) {
      this.saveError = e instanceof Error ? e.message : String(e);
    } finally {
      this.isCheckingUpdate = false;
    }
  }
}
