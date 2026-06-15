import { invoke } from "@tauri-apps/api/core";
import type {
  HealthModuleId,
  HealthModuleInfo,
  HealthSnapshot,
} from "../types/health";
import type { ApiError } from "../types/ring";

function asApiError(error: unknown): ApiError {
  if (typeof error === "string") return { message: error };
  if (error && typeof error === "object" && "message" in error) {
    return { message: String((error as ApiError).message) };
  }
  return { message: "未知错误" };
}

export async function listHealthModules(): Promise<HealthModuleInfo[]> {
  return invoke<HealthModuleInfo[]>("list_health_modules");
}

export async function getHealthSnapshot(): Promise<HealthSnapshot> {
  return invoke<HealthSnapshot>("get_health_snapshot");
}

export async function syncHealthModule(
  moduleId: HealthModuleId,
  mock = false,
): Promise<HealthSnapshot> {
  try {
    return await invoke<HealthSnapshot>("sync_health_module", { moduleId, mock });
  } catch (e) {
    throw asApiError(e);
  }
}

export async function exportHealthCsv(
  moduleId: HealthModuleId,
): Promise<string> {
  try {
    return await invoke<string>("export_health_csv", { moduleId });
  } catch (e) {
    throw asApiError(e);
  }
}

export async function getHealthDbPath(): Promise<string | null> {
  return invoke<string | null>("get_health_db_path");
}
