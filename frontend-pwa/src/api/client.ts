import { reactive } from "vue";
export const requestTelemetry = reactive({
  lastRequestMs: 0,
  lastTraceId: "",
  lastPath: "",
});
export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public traceId = "",
  ) {
    super(message);
  }
}
export async function apiFetch<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const started = performance.now();
  try {
    const headers = new Headers(options.headers);
    if (options.body) headers.set("Content-Type", "application/json");
    const response = await fetch(`/api${path}`, {
      ...options,
      headers,
      credentials: "include",
      signal: options.signal ?? AbortSignal.timeout(90000),
    });
    const body = await response.json().catch(() => {
      throw new ApiError(
        "The server returned an unreadable response.",
        response.status,
      );
    });
    if (body.trace_id || body.error?.trace_id)
      requestTelemetry.lastTraceId = body.trace_id || body.error.trace_id;
    if (!response.ok)
      throw new ApiError(
        body.error?.message || "Request failed. Please try again.",
        response.status,
        body.error?.trace_id,
      );
    return body as T;
  } finally {
    requestTelemetry.lastRequestMs = Math.round(performance.now() - started);
    requestTelemetry.lastPath = path;
  }
}
export const send = (method: string, body: unknown = {}): RequestInit => ({
  method,
  body: JSON.stringify(body),
});
