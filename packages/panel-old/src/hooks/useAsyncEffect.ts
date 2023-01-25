import { useEffect } from "react";

type AsyncEffectCallback = (isMounted: () => boolean) => void;

const useAsyncEffect = (effect: AsyncEffectCallback, deps?: React.DependencyList | undefined) => {
    useEffect(() => {
        let mounted = true;
        Promise.resolve(effect(() => mounted));
        return () => {
            mounted = false;
        };
    }, deps);
};

export default useAsyncEffect;
