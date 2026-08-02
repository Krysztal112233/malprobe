// Mirrors crates/malprobe-vo. Kept in sync by hand for now; the backend
// exposes an OpenAPI spec (utoipa/Scalar at /docs) which can later feed
// openapi-typescript if the API grows.

export type FileStatus = "pending" | "scanning" | "completed" | "failed";
export type FileVerdict = "clean" | "suspicious" | "malicious" | "error";

export interface FileVO {
  id: string;
  sha256: string | null;
  size: number | null;
  mime_type: string | null;
  status: FileStatus;
  verdict: FileVerdict | null;
  malware_name: string | null;
  details: unknown | null;
  error: string | null;
  created_at: string;
  updated_at: string;
  scanned_at: string | null;
}

export interface PageInfo {
  has_next: boolean;
  total: number;
}

export interface PagedResponse<T> {
  items: T[];
  page_info: PageInfo;
}

export interface FileCreateRequest {
  url: string;
}

/** Error envelope returned by the backend on failure (`ApiResponse::error`). */
export interface ApiErrorBody {
  code: number;
  msg: string;
}

/**
 * One engine's scan outcome, as carried in `FileVO.details`.
 *
 * Multi-engine contract: `details` is a free-form JSON blob owned by the
 * scan pipeline; when it holds per-engine results it takes the shape
 * `{ "engines": EngineResult[] }`, e.g.
 *
 * ```json
 * { "engines": [
 *   { "name": "ClamAV", "verdict": "malicious", "malware_name": "Win.Trojan.X" },
 *   { "name": "YARA",   "verdict": "clean" }
 * ] }
 * ```
 */
export interface EngineResult {
  name: string;
  verdict: FileVerdict | null;
  malware_name?: string | null;
}

/** Extracts per-engine results from a `details` blob, or null if absent. */
export function parseEngineResults(details: unknown): EngineResult[] | null {
  if (typeof details !== "object" || details === null) return null;
  const engines = (details as { engines?: unknown }).engines;
  if (!Array.isArray(engines) || engines.length === 0) return null;

  const results: EngineResult[] = [];
  for (const entry of engines) {
    if (typeof entry !== "object" || entry === null) return null;
    const { name, verdict, malware_name } = entry as Record<string, unknown>;
    if (typeof name !== "string") return null;
    results.push({
      name,
      verdict: typeof verdict === "string" ? (verdict as FileVerdict) : null,
      malware_name: typeof malware_name === "string" ? malware_name : null,
    });
  }
  return results;
}
