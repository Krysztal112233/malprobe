import type {
  ApiErrorBody,
  FileCreateRequest,
  FileVO,
  PagedResponse,
} from "./types";

export class ApiError extends Error {
  readonly code: number;

  constructor(body: ApiErrorBody) {
    super(body.msg);
    this.name = "ApiError";
    this.code = body.code;
  }
}

/**
 * API origin, injected at build time via VITE_API_BASE_URL (e.g.
 * "http://192.168.1.10:8000", no trailing slash). Empty string = same
 * origin (dev-server proxy or a production reverse proxy handles routing).
 * Note: a cross-origin base URL requires CORS on the backend
 * (`cors.allow_origins` in malprobe.toml).
 */
const BASE: string = (import.meta.env.VITE_API_BASE_URL ?? "").replace(
  /\/+$/,
  "",
);

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { "Content-Type": "application/json", ...init?.headers },
    ...init,
  });

  if (!res.ok) {
    let message = `HTTP ${res.status}`;
    let code = res.status;
    try {
      const body = (await res.json()) as ApiErrorBody;
      if (body?.msg) {
        message = body.msg;
        code = body.code ?? res.status;
      }
    } catch {
      // Non-JSON error body; fall back to the HTTP status message.
    }
    throw new ApiError({ code, msg: message });
  }

  // ApiResponse flattens successful payloads, so the body IS the payload.
  return (await res.json()) as T;
}

export function listFiles(
  page: number,
  size: number,
): Promise<PagedResponse<FileVO>> {
  return request(`/files?page=${page}&size=${size}`);
}

export function getFile(id: string): Promise<FileVO> {
  return request(`/files/${id}`);
}

export function getFilesByHash(sha256: string): Promise<PagedResponse<FileVO>> {
  return request(`/files/hash/${encodeURIComponent(sha256)}`);
}

export function submitFile(url: string): Promise<FileVO> {
  const body: FileCreateRequest = { url };
  return request("/files", { method: "POST", body: JSON.stringify(body) });
}
