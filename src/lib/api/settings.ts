import { z } from "zod";
import { invokeCommand } from "./transport";
import {
  PreferencesSchema,
  DataInfoSchema,
  AboutInfoSchema,
  CloseActionDecisionSchema,
  type Preferences,
  type DataInfo,
  type AboutInfo,
  type CloseActionDecision,
} from "../contracts/settings";

export async function getSettings(): Promise<Preferences> {
  return await invokeCommand("settings_get", {}, PreferencesSchema);
}

export async function updateSettings(preferences: Preferences): Promise<Preferences> {
  return await invokeCommand("settings_update", { preferences }, PreferencesSchema);
}

export async function getAutostart(): Promise<boolean> {
  return await invokeCommand("settings_autostart_get", {}, z.boolean());
}

export async function setAutostart(enabled: boolean): Promise<void> {
  await invokeCommand("settings_autostart_set", { enabled }, z.void().optional());
}

export async function getDataInfo(): Promise<DataInfo> {
  return await invokeCommand("settings_data_info", {}, DataInfoSchema);
}

export async function purgeData(): Promise<void> {
  await invokeCommand("settings_data_purge", {}, z.void().optional());
}

export async function getAboutInfo(): Promise<AboutInfo> {
  return await invokeCommand("settings_about_info", {}, AboutInfoSchema);
}

export async function evaluateAppClose(force: boolean = false): Promise<CloseActionDecision> {
  return await invokeCommand("app_close_evaluate", { force }, CloseActionDecisionSchema);
}
