/* eslint-disable @typescript-eslint/no-explicit-any */
export class CancelablePromise<T> extends Promise<T> {
    #abortController: AbortController;

    constructor(
        executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason?: any) => void) => void,
        resolveIfCancelled?: T
    ) {
        const abortController = new AbortController();

        const wrapped = (
            executor: (resolve: (value: T | PromiseLike<T>) => void, reject: (reason?: any) => void) => void
        ) => {
            return (resolve: (value: T | PromiseLike<T>) => void, reject: (reason?: any) => void) => {
                abortController.signal.addEventListener("abort", () => {
                    resolveIfCancelled
                        ? resolve(resolveIfCancelled)
                        : reject(new Error("Promise was cancelled with no default resolve value"));
                });
                return executor(resolve, reject);
            };
        };

        super(wrapped(executor));
        this.#abortController = abortController;
    }

    cancel() {
        this.#abortController.abort();
    }
}

export const asyncTimeout = <T extends (...params: any[]) => any>(func: T, timeoutMs: number) => {
    return (...params: Parameters<T>): CancelablePromise<ReturnType<T>> => {
        const promise = new CancelablePromise<ReturnType<T>>((resolve, reject) => {
            const timeout = setTimeout(() => {
                reject(new Error("Promise timed out"));
            }, timeoutMs);
            func(...params)
                .then((result: ReturnType<T>) => {
                    clearTimeout(timeout);
                    resolve(result);
                })
                .catch((err: any) => {
                    clearTimeout(timeout);
                    reject(err);
                });
        });
        return promise;
    };
};
