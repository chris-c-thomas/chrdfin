/** Discriminated union for error handling without exceptions. */
export type Result<T, E = Error> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

/** Progress event for long-running computations (received via Tauri events). */
export interface ProgressEvent {
  readonly phase: string;
  readonly current: number;
  readonly total: number;
  readonly message?: string;
}

/** ISO date string in YYYY-MM-DD format. */
export type ISODateString = string;

/** ISO datetime string in full ISO 8601 format. */
export type ISODateTimeString = string;

/** UUID string. */
export type UUID = string;
