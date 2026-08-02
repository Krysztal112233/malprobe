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

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, {
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
