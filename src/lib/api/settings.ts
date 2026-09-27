import { z } from "zod";
import { invokeCommand } from "./transport";
import {
  PreferencesSchema,
  DataInfoSchema,
  AboutInfoSchema,
  DiagnosticPathsSchema,
  type Preferences,
  type DataInfo,
  type AboutInfo,
  type DiagnosticPaths,
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
  await invokeCommand("settings_autostart_set", { enabled }, z.void().nullable());
}

export async function getDataInfo(): Promise<DataInfo> {
  return await invokeCommand("settings_data_info", {}, DataInfoSchema);
}

export async function purgeData(): Promise<void> {
  await invokeCommand("settings_data_purge", {}, z.void().nullable());
}

export async function getAboutInfo(): Promise<AboutInfo> {
  return await invokeCommand("settings_about_info", {}, AboutInfoSchema);
}

export async function getDiagnosticPaths(): Promise<DiagnosticPaths> {
  return await invokeCommand("settings_diagnostic_paths", {}, DiagnosticPathsSchema);
}

export async function openLogFolder(): Promise<void> {
  await invokeCommand("settings_open_log_folder", {}, z.void().nullable());
}
