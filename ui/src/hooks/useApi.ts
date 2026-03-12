import { useState, useEffect, useCallback } from 'react';
import { apiGet, apiPost, apiPut, apiDelete } from '../lib/api';

export interface UseApiOptions<T> {
  /** Initial data */
  initialData?: T;
  /** Auto-fetch on mount (default: true) */
  autoFetch?: boolean;
  /** Callback on error */
  onError?: (error: Error) => void;
}

export interface UseApiState<T> {
  data: T | undefined;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

/**
 * Custom hook for fetching data from API
 * Reduces boilerplate in components
 */
export function useApi<T>(path: string, options: UseApiOptions<T> = {}): UseApiState<T> {
  const { initialData, autoFetch = true, onError } = options;
  
  const [data, setData] = useState<T | undefined>(initialData);
  const [loading, setLoading] = useState(autoFetch);
  const [error, setError] = useState<Error | null>(null);

  const refetch = useCallback(async () => {
    setLoading(true);
    setError(null);
    
    try {
      const response = await apiGet(path);
      if (!response.ok) {
        throw new Error(`API error: ${response.status} ${response.statusText}`);
      }
      const result = await response.json();
      setData(result);
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      setError(err);
      onError?.(err);
    } finally {
      setLoading(false);
    }
  }, [path, onError]);

  useEffect(() => {
    if (autoFetch) {
      refetch();
    }
  }, [autoFetch, refetch]);

  return { data, loading, error, refetch };
}

/**
 * Hook for mutating data (POST, PUT, DELETE)
 */
export function useApiMutation<TRequest, TResponse>(
  method: 'POST' | 'PUT' | 'DELETE',
  path: string,
  options: {
    onSuccess?: (data: TResponse) => void;
    onError?: (error: Error) => void;
  } = {}
) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (body?: TRequest): Promise<TResponse | null> => {
    setLoading(true);
    setError(null);
    
    try {
      let response;
      if (method === 'POST') {
        response = await apiPost(path, body);
      } else if (method === 'PUT') {
        response = await apiPut(path, body);
      } else {
        response = await apiDelete(path);
      }
      
      if (!response.ok) {
        throw new Error(`API error: ${response.status} ${response.statusText}`);
      }
      
      const result = await response.json();
      options.onSuccess?.(result);
      return result;
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      setError(err);
      options.onError?.(err);
      return null;
    } finally {
      setLoading(false);
    }
  }, [method, path, options]);

  return { mutate, loading, error };
}

/**
 * Hook for creating resources (POST)
 */
export function useCreate<TRequest, TResponse>(path: string) {
  return useApiMutation<TRequest, TResponse>('POST', path);
}

/**
 * Hook for updating resources (PUT)
 */
export function useUpdate<TRequest, TResponse>(path: string) {
  return useApiMutation<TRequest, TResponse>('PUT', path);
}

/**
 * Hook for deleting resources (DELETE)
 */
export function useDelete<TResponse>(path: string) {
  return useApiMutation<null, TResponse>('DELETE', path);
}
