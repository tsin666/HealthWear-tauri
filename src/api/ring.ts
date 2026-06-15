import { invoke } from "@tauri-apps/api/core";
import type {
  ApiError,
  ConnectionInfo,
  ScannedDevice,
  WisdomModeItem,
  WisdomState,
} from "../types/ring";

function asApiError(error: unknown): ApiError {
  if (typeof error === "string") return { message: error };
  if (error && typeof error === "object" && "message" in error) {
    return { message: String((error as ApiError).message) };
  }
  return { message: "未知错误" };
}

export async function getConnection(): Promise<ConnectionInfo> {
  return invoke<ConnectionInfo>("get_connection");
}

export async function scanDevices(): Promise<ScannedDevice[]> {
  try {
    return await invoke<ScannedDevice[]>("scan_devices");
  } catch (e) {
    throw asApiError(e);
  }
}

export async function connectDevice(deviceId: string): Promise<void> {
  try {
    await invoke("connect_device", { deviceId });
  } catch (e) {
    throw asApiError(e);
  }
}

export async function disconnectDevice(): Promise<void> {
  try {
    await invoke("disconnect_device");
  } catch (e) {
    throw asApiError(e);
  }
}

export async function listWisdomModes(): Promise<WisdomModeItem[]> {
  return invoke<WisdomModeItem[]>("list_wisdom_modes");
}

export async function getWisdomState(): Promise<WisdomState> {
  return invoke<WisdomState>("get_wisdom_state");
}

export async function setWisdomMode(
  protocolIndex: number,
  enabled: boolean,
): Promise<WisdomState> {
  try {
    return await invoke<WisdomState>("set_wisdom_mode", {
      protocolIndex,
      enabled,
    });
  } catch (e) {
    throw asApiError(e);
  }
}
