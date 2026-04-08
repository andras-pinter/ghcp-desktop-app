/** Reactive network status store using Svelte 5 runes. */

let online = $state(typeof navigator !== "undefined" ? navigator.onLine : true);
let unlisteners: (() => void)[] = [];

/** Initialize network status monitoring. Call once on app startup. */
export function initNetwork(): void {
  // Clean up previous listeners (HMR)
  destroyNetwork();

  const handleOnline = () => {
    online = true;
  };
  const handleOffline = () => {
    online = false;
  };

  window.addEventListener("online", handleOnline);
  window.addEventListener("offline", handleOffline);

  unlisteners = [
    () => window.removeEventListener("online", handleOnline),
    () => window.removeEventListener("offline", handleOffline),
  ];
}

/** Clean up event listeners. */
export function destroyNetwork(): void {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  unlisteners = [];
}

/** Reactive getter for network status. */
export function getNetwork() {
  return {
    get isOnline() {
      return online;
    },
  };
}
