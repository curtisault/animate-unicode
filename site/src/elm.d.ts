// What vite-plugin-elm hands back for an imported .elm file.
declare module "*.elm" {
  export const Elm: {
    Main: {
      init(options: { node: HTMLElement | null; flags?: unknown }): {
        ports: Record<string, { subscribe(cb: (value: unknown) => void): void; send(value: unknown): void }>;
      };
    };
  };
}
