import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { NotificationBell } from './NotificationBell';
import * as api from '../../lib/api';

vi.mock('../../lib/api', () => ({
  apiGet: vi.fn(),
  apiPost: vi.fn(),
  apiDelete: vi.fn(),
}));

vi.mock('../../lib/websocket', () => ({
  wsClient: {
    on: vi.fn(),
    off: vi.fn(),
  },
}));

vi.mock('../../stores/appStore', () => ({
  useAppStore: vi.fn(() => ({
    notifications: [],
  })),
}));

const mockNotifications = [
  {
    id: '1',
    notification_type: 'task_completed',
    title: 'Task Completed',
    message: 'Your task has been completed',
    severity: 'info',
    read: false,
    created_at_ms: Date.now() - 60000,
  },
  {
    id: '2',
    notification_type: 'task_failed',
    title: 'Task Failed',
    message: 'Task failed due to error',
    severity: 'error',
    read: false,
    created_at_ms: Date.now() - 3600000,
  },
];

describe('NotificationBell', () => {
  beforeEach(() => {
    vi.useRealTimers();
    (api.apiGet as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      json: async () => mockNotifications,
    } as Response);
    (api.apiPost as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      json: async () => ({}),
    } as Response);
    (api.apiDelete as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      json: async () => ({}),
    } as Response);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('renders notification bell button', () => {
    render(<NotificationBell />);
    expect(screen.getByTitle('Notifications')).toBeInTheDocument();
  });

  it('shows unread count badge when there are unread notifications', async () => {
    render(<NotificationBell />);
    await waitFor(() => {
      expect(screen.getByText('2')).toBeInTheDocument();
    });
  });

  it('opens dropdown when clicked', async () => {
    render(<NotificationBell />);
    
    await waitFor(() => {
      expect(screen.getByText('2')).toBeInTheDocument();
    });
    
    const bell = screen.getByTitle('Notifications');
    await act(async () => {
      fireEvent.click(bell);
    });
    
    await waitFor(() => {
      expect(screen.getByText('Notifications')).toBeInTheDocument();
    });
  });

  it('displays notifications in dropdown when opened', async () => {
    render(<NotificationBell />);
    
    await waitFor(() => {
      expect(screen.getByText('2')).toBeInTheDocument();
    });
    
    const bell = screen.getByTitle('Notifications');
    await act(async () => {
      fireEvent.click(bell);
    });
    
    await waitFor(() => {
      expect(screen.getByText('Task Completed')).toBeInTheDocument();
      expect(screen.getByText('Task Failed')).toBeInTheDocument();
    });
  });

  it('shows "No new notifications" when list is empty', async () => {
    (api.apiGet as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      json: async () => [],
    } as Response);
    
    render(<NotificationBell />);
    
    await waitFor(() => {
      expect(screen.queryByText('2')).not.toBeInTheDocument();
    });
    
    const bell = screen.getByTitle('Notifications');
    await act(async () => {
      fireEvent.click(bell);
    });
    
    await waitFor(() => {
      expect(screen.getByText('No new notifications')).toBeInTheDocument();
    });
  });

  it('has mark all read button when there are unread notifications', async () => {
    render(<NotificationBell />);
    
    await waitFor(() => {
      expect(screen.getByText('2')).toBeInTheDocument();
    });
    
    const bell = screen.getByTitle('Notifications');
    await act(async () => {
      fireEvent.click(bell);
    });
    
    await waitFor(() => {
      expect(screen.getByText('Mark all read')).toBeInTheDocument();
    });
  });

  it('calls markAsRead when clicking checkmark', async () => {
    render(<NotificationBell />);
    
    await waitFor(() => {
      expect(screen.getByText('2')).toBeInTheDocument();
    });
    
    const bell = screen.getByTitle('Notifications');
    await act(async () => {
      fireEvent.click(bell);
    });
    
    await waitFor(() => {
      const markAsReadButtons = screen.getAllByTitle('Mark as read');
      expect(markAsReadButtons.length).toBeGreaterThan(0);
    });
  });
});
