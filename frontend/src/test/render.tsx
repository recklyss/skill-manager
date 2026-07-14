import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, type RenderOptions } from "@testing-library/react";
import { type ReactElement, type ReactNode } from "react";
import { MemoryRouter } from "react-router-dom";

import { ThemeProvider } from "../lib/useTheme";
import { ToastProvider } from "../components/Toast";
import { UiTooltipProvider } from "../components/ui/UiTooltipProvider";
import { LocaleProvider } from "../i18n";

export function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
}

export function renderWithAppProviders(
  ui: ReactElement,
  {
    route = "/",
    queryClient = createTestQueryClient(),
    ...renderOptions
  }: RenderOptions & {
    route?: string;
    queryClient?: QueryClient;
  } = {},
) {
  const result = render(ui, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <ThemeProvider>
        <QueryClientProvider client={queryClient}>
          <LocaleProvider>
            <UiTooltipProvider delayDuration={0} skipDelayDuration={0}>
              <ToastProvider>
                <MemoryRouter initialEntries={[route]}>{children}</MemoryRouter>
              </ToastProvider>
            </UiTooltipProvider>
          </LocaleProvider>
        </QueryClientProvider>
      </ThemeProvider>
    ),
    ...renderOptions,
  });

  return { ...result, queryClient };
}

export function renderWithRouter(
  ui: ReactElement,
  {
    route = "/",
    ...renderOptions
  }: RenderOptions & {
    route?: string;
  } = {},
) {
  return render(ui, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <ThemeProvider>
        <MemoryRouter initialEntries={[route]}>{children}</MemoryRouter>
      </ThemeProvider>
    ),
    ...renderOptions,
  });
}

export function stubDesktopMatchMedia(): void {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    configurable: true,
    value: (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
      addListener: () => undefined,
      removeListener: () => undefined,
      dispatchEvent: () => false,
    }),
  });
}
