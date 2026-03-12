import { useState, useEffect, useRef } from 'react';
import { apiGet, apiPost, apiDelete } from '../../lib/api';
import { useAppStore } from '../../stores/appStore';

interface Notification {
  id: string;
  notification_type: string;
  title: string;
  message: string;
  severity: string;
  read: boolean;
  created_at_ms: number;
  data?: Record<string, unknown>;
}

export function NotificationBell() {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const storeNotifications = useAppStore((state) => state.notifications);

  useEffect(() => {
    loadNotifications();
    const interval = setInterval(loadNotifications, 10000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (storeNotifications.length > 0) {
      setNotifications(storeNotifications);
      setUnreadCount(storeNotifications.filter(n => !n.read).length);
    }
  }, [storeNotifications]);

  useEffect(() => {
    if (storeNotifications.length > 0) {
      setNotifications((prev) => {
        const existingIds = new Set(prev.map((n) => n.id));
        const newNotifs = storeNotifications.filter((n) => !existingIds.has(n.id));
        if (newNotifs.length > 0) {
          return [...newNotifs, ...prev];
        }
        return prev;
      });
      setUnreadCount(storeNotifications.filter((n) => !n.read).length);
    }
  }, [storeNotifications]);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  async function loadNotifications() {
    try {
      const response = await apiGet('/api/v1/notifications?include_read=false');
      if (response.ok) {
        const data = await response.json();
        setNotifications(data);
        setUnreadCount(data.length);
      }
    } catch (error) {
      console.error('Failed to load notifications:', error);
    }
  }

  async function markAsRead(id: string) {
    try {
      await apiPost(`/api/v1/notifications/${id}/read`, {});
      loadNotifications();
    } catch (error) {
      console.error('Failed to mark as read:', error);
    }
  }

  async function markAllRead() {
    try {
      await apiPost('/api/v1/notifications/read-all', {});
      loadNotifications();
    } catch (error) {
      console.error('Failed to mark all as read:', error);
    }
  }

  async function deleteNotification(id: string) {
    try {
      await apiDelete(`/api/v1/notifications/${id}`);
      loadNotifications();
    } catch (error) {
      console.error('Failed to delete notification:', error);
    }
  }

  function getSeverityColor(severity: string) {
    switch (severity) {
      case 'error': return 'text-red-500';
      case 'warning': return 'text-amber-500';
      case 'info': return 'text-blue-500';
      default: return 'text-[var(--color-text-muted)]';
    }
  }

  function getSeverityIcon(severity: string) {
    switch (severity) {
      case 'error':
        return <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-red-500"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg>;
      case 'warning':
        return <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-amber-500"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>;
      case 'info':
        return <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-blue-500"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>;
      default:
        return <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-[var(--color-text-muted)]"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path><path d="M13.73 21a2 2 0 0 1-3.46 0"></path></svg>;
    }
  }

  function formatTime(timestamp: number) {
    const diff = Date.now() - timestamp;
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return new Date(timestamp).toLocaleDateString();
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="p-2 hover:bg-[var(--color-muted)] rounded-lg transition-colors relative"
        title="Notifications"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-[var(--color-text)]">
          <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
          <path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
        </svg>
        {unreadCount > 0 && (
          <span className="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center font-medium">
            {unreadCount > 9 ? '9+' : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="absolute right-0 top-full mt-2 w-80 bg-[var(--color-panel)] border border-[var(--color-border)] rounded-xl shadow-xl z-50 max-h-96 overflow-hidden flex flex-col">
          <div className="p-3 border-b border-[var(--color-border)] flex items-center justify-between">
            <span className="font-semibold text-[var(--color-text)]">Notifications</span>
            {unreadCount > 0 && (
              <button
                onClick={markAllRead}
                className="text-xs text-[#4248f1] hover:underline"
              >
                Mark all read
              </button>
            )}
          </div>
          
          <div className="overflow-y-auto flex-1">
            {notifications.length === 0 ? (
              <div className="p-4 text-center text-[var(--color-text-muted)] text-sm">
                No new notifications
              </div>
            ) : (
              notifications.map((notification) => (
                <div
                  key={notification.id}
                  className={`p-3 border-b border-[var(--color-border)] last:border-b-0 hover:bg-[var(--color-muted)]/50 ${!notification.read ? 'bg-[var(--color-muted)]/30' : ''}`}
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        {getSeverityIcon(notification.severity)}
                        <span className={`text-sm font-medium ${getSeverityColor(notification.severity)}`}>
                          {notification.title}
                        </span>
                        {!notification.read && (
                          <span className="w-2 h-2 bg-[#4248f1] rounded-full" />
                        )}
                      </div>
                      <p className="text-xs text-[var(--color-text-muted)] truncate mt-1">
                        {notification.message}
                      </p>
                      <span className="text-xs text-[var(--color-text-muted)]">
                        {formatTime(notification.created_at_ms)}
                      </span>
                    </div>
                    <div className="flex items-center gap-1">
                      {!notification.read && (
                        <button
                          onClick={() => markAsRead(notification.id)}
                          className="p-1 hover:bg-[var(--color-muted)] rounded"
                          title="Mark as read"
                        >
                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                        </button>
                      )}
                      <button
                        onClick={() => deleteNotification(notification.id)}
                        className="p-1 hover:bg-[var(--color-muted)] rounded text-[var(--color-text-muted)]"
                        title="Delete"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                      </button>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}
