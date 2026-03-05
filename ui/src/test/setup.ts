import '@testing-library/jest-dom';

import { vi } from 'vitest';

vi.mock('zustand', () => ({
  create: () => ({}),
  useStore: () => ({}),
}));

vi.mock('@tanstack/react-query', () => ({
  useQuery: () => ({ data: undefined, isLoading: false }),
  useMutation: () => ({ mutate: () => {}, isLoading: false }),
  QueryClient: class QueryClient {
    constructor() {}
    setDefaultOptions() {}
  },
  QueryClientProvider: ({ children }: { children: React.ReactNode }) => children,
}));

vi.mock('socket.io-client', () => ({
  io: () => ({
    on: () => {},
    off: () => {},
    emit: () => {},
    disconnect: () => {},
  }),
}));

window.matchMedia = window.matchMedia || function() {
  return {
    matches: false,
    media: '',
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  };
};
