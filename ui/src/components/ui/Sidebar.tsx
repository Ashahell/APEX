import clsx from 'clsx';

interface SidebarProps {
  activeTab: 'chat' | 'skills' | 'files' | 'kanban' | 'settings';
  onTabChange: (tab: 'chat' | 'skills' | 'files' | 'kanban' | 'settings') => void;
}

const tabs = [
  { id: 'chat' as const, label: 'Chat', icon: '💬' },
  { id: 'skills' as const, label: 'Skills', icon: '⚡' },
  { id: 'files' as const, label: 'Files', icon: '📁' },
  { id: 'kanban' as const, label: 'Board', icon: '📋' },
  { id: 'settings' as const, label: 'Settings', icon: '⚙️' },
];

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  return (
    <aside className="w-16 border-r flex flex-col items-center py-4 gap-2">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onTabChange(tab.id)}
          className={clsx(
            'w-12 h-12 rounded-lg flex items-center justify-center text-lg transition-colors',
            activeTab === tab.id
              ? 'bg-primary text-primary-foreground'
              : 'hover:bg-muted'
          )}
          title={tab.label}
        >
          {tab.icon}
        </button>
      ))}
    </aside>
  );
}
