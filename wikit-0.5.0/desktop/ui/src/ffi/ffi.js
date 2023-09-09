const invoke = window.__TAURI__.invoke;
const tauri = window.__TAURI__;

export async function ffiHello(name) {
    return await invoke("ffi_hello", { name });
}

export async function startPreviewServer(dir) {
    return await invoke("start_preview_server", { dir });
}

export async function stopPreviewServer() {
    return await invoke("stop_preview_server");
}

export async function open() {
    return await tauri.dialog.open({ directory: true });
}

export async function isPreviewServerUp() {
  return await invoke("is_preview_server_up")
}

export async function lookUp(dictId, word) {
  return await invoke("lookup", { "dictid": dictId, "word": word })
}

export async function getDictList() {
  return await invoke("get_dict_list")
}
