import { useState } from 'react';
import clsx from 'clsx';

export type AppTab = 
  // Top-level
  | 'chat' | 'board' | 'workflows' | 'settings' | 'theme'
  // Memory
  | 'memory' | 'memoryStats' | 'narrative'
  // Skills
  | 'skills' | 'marketplace' | 'deep'
  // Work
  | 'files' | 'channels' | 'journal' | 'audit' | 'consequences'
  // System
  | 'metrics' | 'monitoring' | 'health' | 'vm'
  // Security
  | 'totp' | 'clients'
  // Integrations
  | 'adapters' | 'webhooks' | 'social'
  // Agent
  | 'soul' | 'autonomy' | 'governance';

interface SidebarItem {
  id: AppTab;
  label: string;
}

interface SidebarGroup {
  id: string;
  label: string;
  icon: string;
  items: SidebarItem[];
}

// Matching AgentZero sidebar structure
const SIDEBAR_GROUPS: SidebarGroup[] = [
  {
    id: 'main',
    label: 'Main',
    icon: '◈',
    items: [
      { id: 'chat', label: 'Chat' },
      { id: 'board', label: 'Board' },
      { id: 'workflows', label: 'Workflows' },
      { id: 'settings', label: 'Settings' },
      { id: 'theme', label: 'Theme' },
    ],
  },
  {
    id: 'memory',
    label: 'Memory',
    icon: '▓',
    items: [
      { id: 'memory', label: 'Memory' },
      { id: 'memoryStats', label: 'Stats' },
      { id: 'narrative', label: 'Narrative' },
    ],
  },
  {
    id: 'skills',
    label: 'Skills',
    icon: '⚡',
    items: [
      { id: 'skills', label: 'Registry' },
      { id: 'marketplace', label: 'Marketplace' },
      { id: 'deep', label: 'Deep Tasks' },
    ],
  },
  {
    id: 'work',
    label: 'Work',
    icon: '▤',
    items: [
      { id: 'files', label: 'Files' },
      { id: 'channels', label: 'Channels' },
      { id: 'journal', label: 'Journal' },
      { id: 'audit', label: 'Audit' },
      { id: 'consequences', label: 'Preview' },
    ],
  },
  {
    id: 'system',
    label: 'System',
    icon: '▣',
    items: [
      { id: 'metrics', label: 'Metrics' },
      { id: 'monitoring', label: 'Monitor' },
      { id: 'health', label: 'Health' },
      { id: 'vm', label: 'VMs' },
    ],
  },
  {
    id: 'security',
    label: 'Security',
    icon: '§',
    items: [
      { id: 'totp', label: '2FA' },
      { id: 'clients', label: 'Clients' },
    ],
  },
  {
    id: 'integrations',
    label: 'Integrations',
    icon: '◈',
    items: [
      { id: 'adapters', label: 'Adapters' },
      { id: 'webhooks', label: 'Webhooks' },
      { id: 'social', label: 'Social' },
    ],
  },
  {
    id: 'agent',
    label: 'Agent',
    icon: '◆',
    items: [
      { id: 'soul', label: 'Identity' },
      { id: 'autonomy', label: 'Autonomy' },
      { id: 'governance', label: 'Governance' },
    ],
  },
];

// Quick action icons - AgentZero style with indigo
const QUICK_ACTIONS = [
  { id: 'chat', label: 'New Chat', action: 'newChat' },
  { id: 'memory', label: 'Memory', action: 'memory' },
  { id: 'settings', label: 'Settings', action: 'settings' },
];

// SVG Icon component
function NavIcon({ tab, className = "w-5 h-5" }: { tab: AppTab; className?: string }) {
  const icons: Record<string, React.ReactNode> = {
    chat: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" /></svg>,
    board: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2" /></svg>,
    workflows: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" /></svg>,
    settings: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /></svg>,
    theme: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" /></svg>,
    memory: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>,
    memoryStats: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" /></svg>,
    narrative: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" /></svg>,
    skills: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>,
    marketplace: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 100 4 2 2 0 000-4zm-8 2a2 2 0 11-4 0 2 2 0 014 0z" /></svg>,
    deep: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" /></svg>,
    files: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>,
    channels: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 8h2a2 2 0 012 2v6a2 2 0 01-2 2h-2v4l-4-4H9a1.994 1.994 0 01-1.414-.586m0 0L11 14h4a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2v4l.586-.586z" /></svg>,
    journal: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>,
    audit: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>,
    consequences: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>,
    metrics: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" /></svg>,
    monitoring: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" /></svg>,
    health: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" /></svg>,
    vm: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" /></svg>,
    totp: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg>,
    clients: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" /></svg>,
    adapters: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>,
    webhooks: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>,
    social: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>,
    soul: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" /></svg>,
    autonomy: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" /></svg>,
    governance: <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 6l3 1m0 0l-3 9a5.002 5.002 0 006.001 0M6 7l3 9M6 7l6-2m6 2l3-1m-3 1l-3 9a5.002 5.002 0 006.001 0M18 7l3 9m-3-9l-6-2m0-2v2m0 16V5m0 16H9m3 0h3" /></svg>,
  };
  return <span className={className}>{icons[tab] || icons.chat}</span>;
}

interface SidebarProps {
  activeTab: AppTab;
  onTabChange: (tab: AppTab) => void;
  collapsed?: boolean;
}

export function Sidebar({ activeTab, onTabChange, collapsed = false }: SidebarProps) {
  const [expandedGroup, setExpandedGroup] = useState<string | null>(null);

  const toggleGroup = (groupId: string) => {
    setExpandedGroup(prev => prev === groupId ? null : groupId);
  };

  return (
    <>
      {/* Desktop Sidebar - AgentZero Style */}
      <aside className={clsx(
        "hidden md:flex flex-col border-r bg-card transition-all duration-200 shrink-0",
        collapsed ? "w-14" : "w-56"
      )}>
        {/* Top Section - Quick Actions (AgentZero style) */}
        <div className="border-b border-border p-2">
          <div className={clsx(
            "flex gap-1",
            collapsed ? "flex-col" : ""
          )}>
            {QUICK_ACTIONS.map((action) => (
              <button
                key={action.id}
                onClick={() => action.action === 'newChat' ? onTabChange('chat') : action.action === 'settings' ? onTabChange('settings') : onTabChange('memory')}
                className={clsx(
                  "flex-1 flex items-center justify-center rounded-xl transition-colors p-2",
                  "hover:bg-[#4248f1]/20",
                  activeTab === action.id ? "bg-[#4248f1]/20 text-[#4248f1]" : "text-[var(--color-text-muted)]"
                )}
                title={action.label}
              >
                <NavIcon tab={action.id as AppTab} className="w-5 h-5" />
              </button>
            ))}
          </div>
        </div>

        {/* Scrollable Content */}
        <div className="flex-1 overflow-y-auto p-2">
          {!collapsed && (
            <div className="space-y-1">
              {/* Main Navigation */}
              {SIDEBAR_GROUPS[0].items.map((item) => (
                <button
                  key={item.id}
                  onClick={() => onTabChange(item.id)}
                  className={clsx(
                    "w-full flex items-center gap-3 px-3 py-2 rounded-xl text-sm transition-colors",
                    activeTab === item.id
                      ? "bg-[#4248f1]/20 text-[#4248f1] font-medium"
                      : "hover:bg-[#4248f1]/10 text-[var(--color-text-muted)]"
                  )}
                >
                  <NavIcon tab={item.id as AppTab} className="w-5 h-5" />
                  <span>{item.label}</span>
                </button>
              ))}

              {/* Collapsible Groups */}
              {SIDEBAR_GROUPS.slice(1).map((group) => (
                <div key={group.id} className="mt-2">
                  <button
                    onClick={() => toggleGroup(group.id)}
                    className={clsx(
                      "w-full flex items-center justify-between px-3 py-2 rounded-xl text-sm transition-colors",
                      "hover:bg-[#4248f1]/10 text-[var(--color-text-muted)]",
                      expandedGroup === group.id && "bg-[#4248f1]/10"
                    )}
                  >
                    <span className="flex items-center gap-2">
                      <NavIcon tab={group.items[0].id as AppTab} className="w-4 h-4" />
                      <span>{group.label}</span>
                    </span>
                    <span className={clsx(
                      "text-xs transition-transform",
                      expandedGroup === group.id && "rotate-90"
                    )}>
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                      </svg>
                    </span>
                  </button>
                  
                  {expandedGroup === group.id && (
                    <div className="ml-4 mt-1 space-y-1">
                      {group.items.map((item) => (
                        <button
                          key={item.id}
                          onClick={() => onTabChange(item.id)}
                          className={clsx(
                            "w-full flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm transition-colors",
                            activeTab === item.id
                              ? "bg-[#4248f1]/20 text-[#4248f1] font-medium"
                              : "hover:bg-[#4248f1]/10 text-[var(--color-text-muted)]"
                          )}
                        >
                          <NavIcon tab={item.id as AppTab} className="w-4 h-4" />
                          <span>{item.label}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Collapsed View */}
          {collapsed && (
            <div className="space-y-1">
              {SIDEBAR_GROUPS.map((group) => (
                <div key={group.id} className="relative">
                  <button
                    onClick={() => toggleGroup(group.id)}
                    className={clsx(
                      "w-full flex items-center justify-center p-2 rounded-xl transition-colors",
                      expandedGroup === group.id
                        ? "bg-[#4248f1]/20 text-[#4248f1]"
                        : "hover:bg-[#4248f1]/10 text-[var(--color-text-muted)]"
                    )}
                    title={group.label}
                  >
                    <NavIcon tab={group.items[0].id as AppTab} className="w-5 h-5" />
                  </button>
                  
                  {expandedGroup === group.id && (
                    <div className="absolute left-full top-0 ml-1 bg-[var(--color-panel)] border border-border rounded-xl shadow-lg py-1 min-w-[140px] z-50">
                      <div className="px-3 py-1 text-xs font-semibold text-[var(--color-text-muted)] border-b border-border">
                        {group.label}
                      </div>
                      {group.items.map((item) => (
                        <button
                          key={item.id}
                          onClick={() => {
                            onTabChange(item.id);
                            setExpandedGroup(null);
                          }}
                          className={clsx(
                            "w-full px-3 py-2 text-left text-sm hover:bg-[#4248f1]/10 flex items-center gap-2",
                            activeTab === item.id && "bg-[#4248f1]/20 text-[#4248f1]"
                          )}
                        >
                          <NavIcon tab={item.id as AppTab} className="w-4 h-4" />
                          <span>{item.label}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Bottom Section - Version Info (AgentZero style) */}
        {!collapsed && (
          <div className="border-t border-border p-3">
            <div className="text-xs text-[var(--color-text-muted)]">
              <div className="font-medium" style={{ color: '#4248f1' }}>APEX</div>
              <div className="opacity-70">v1.3.2</div>
            </div>
          </div>
        )}
      </aside>
      
      {/* Mobile Bottom Nav */}
      <nav className="md:hidden fixed bottom-0 left-0 right-0 border-t border-border bg-[var(--color-background)] flex items-center justify-around py-2 z-50">
        {SIDEBAR_GROUPS[0].items.slice(0, 4).map((item) => (
          <button
            key={item.id}
            onClick={() => onTabChange(item.id)}
            className={clsx(
              "flex flex-col items-center justify-center p-2 rounded-xl transition-colors",
              activeTab === item.id
                ? "text-[#4248f1]"
                : "text-[var(--color-text-muted)] hover:text-[var(--color-foreground)]"
            )}
          >
            <NavIcon tab={item.id as AppTab} className="w-5 h-5" />
            <span className="text-xs mt-1">{item.label}</span>
          </button>
        ))}
        <button
          onClick={() => onTabChange('settings')}
          className={clsx(
            "flex flex-col items-center justify-center p-2 rounded-xl transition-colors",
            activeTab === 'settings'
              ? "text-[#4248f1]"
              : "text-[var(--color-text-muted)] hover:text-[var(--color-foreground)]"
          )}
        >
          <NavIcon tab="settings" className="w-5 h-5" />
          <span className="text-xs mt-1">More</span>
        </button>
      </nav>
    </>
  );
}
