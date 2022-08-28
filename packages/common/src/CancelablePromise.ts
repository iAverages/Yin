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
