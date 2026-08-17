export class ApiRequestError extends Error {
    constructor(
        public readonly status: number,
        message: string,
    ) {
        super(message);
        this.name = "ApiRequestError";
    }
}

// Client errors (auth, not found, validation) fail the same way again; timeouts and rate limits may not.
export const isRetryableError = (error: Error) =>
    !(error instanceof ApiRequestError) ||
    error.status >= 500 ||
    error.status === 408 ||
    error.status === 429;
