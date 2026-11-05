/// <reference types="vite/client" />

/**
 * `package.json` version, injected at build time by `vite.config.ts`
 * (`define: { __APP_VERSION__: ... }`). Keeps the frontend fallback version in
 * lockstep with the shipped binary instead of a hand-edited literal.
 */
declare const __APP_VERSION__: string;
