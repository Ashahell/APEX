import { useState, useEffect } from 'react';

interface Story {
  id: string;
  title: string;
  setting: string;
  turn_count: number;
  location: string;
  created_at: number;
  updated_at: number;
}

interface StoryListProps {
  onSelectStory?: (storyId: string) => void;
  onCreateNew?: () => void;
}

// Setting icons
const SETTING_ICONS: Record<string, string> = {
  fantasy: '⚔️',
  scifi: '🚀',
  horror: '👻',
  mystery: '🔍',
  western: '🤠',
  modern: '🌆',
};

export function StoryList({ onSelectStory, onCreateNew }: StoryListProps) {
  const [stories, setStories] = useState<Story[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadStories();
  }, []);

  const loadStories = async () => {
    setIsLoading(true);
    try {
      const res = await fetch('/api/v1/stories', {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      
      if (res.ok) {
        const data = await res.json();
        setStories(data.stories || []);
      } else {
        // Demo stories for when API not available
        setStories([
          { id: 'demo-1', title: "The Dragon's Lair", setting: 'fantasy', turn_count: 5, location: 'Forest', created_at: Date.now(), updated_at: Date.now() },
          { id: 'demo-2', title: 'Space Station Omega', setting: 'scifi', turn_count: 12, location: 'Airlock', created_at: Date.now(), updated_at: Date.now() },
        ]);
      }
    } catch (err) {
      console.error('Failed to load stories:', err);
      setStories([
        { id: 'demo-1', title: "The Dragon's Lair", setting: 'fantasy', turn_count: 5, location: 'Forest', created_at: Date.now(), updated_at: Date.now() },
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString();
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            📖 Your Stories
          </h2>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Continue your adventures or start something new
          </p>
        </div>
        <button
          onClick={onCreateNew}
          className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 flex items-center gap-2"
        >
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          New Story
        </button>
      </div>

      {/* Stories Grid */}
      {stories.length === 0 ? (
        <div className="text-center py-12">
          <div className="text-6xl mb-4">📚</div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            No stories yet
          </h3>
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            Start your first adventure to begin your journey
          </p>
          <button
            onClick={onCreateNew}
            className="px-6 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700"
          >
            Create Your First Story
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {stories.map((story) => (
            <button
              key={story.id}
              onClick={() => onSelectStory?.(story.id)}
              className="p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-left hover:border-indigo-500 dark:hover:border-indigo-500 hover:shadow-md transition-all"
            >
              <div className="flex items-start justify-between mb-2">
                <div className="text-2xl">
                  {SETTING_ICONS[story.setting] || '📖'}
                </div>
                <span className="text-xs text-gray-500 dark:text-gray-400">
                  Turn {story.turn_count}
                </span>
              </div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-1">
                {story.title}
              </h3>
              <div className="text-sm text-gray-500 dark:text-gray-400">
                📍 {story.location}
              </div>
              <div className="text-xs text-gray-400 dark:text-gray-500 mt-2">
                {formatDate(story.created_at)}
              </div>
            </button>
          ))}
        </div>
      )}

      {/* Quick Actions */}
      <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
        <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
          Quick Start Templates
        </h3>
        <div className="flex flex-wrap gap-2">
          {[
            { id: 'fantasy', icon: '⚔️', label: 'Fantasy' },
            { id: 'scifi', icon: '🚀', label: 'Sci-Fi' },
            { id: 'horror', icon: '👻', label: 'Horror' },
            { id: 'mystery', icon: '🔍', label: 'Mystery' },
            { id: 'western', icon: '🤠', label: 'Western' },
            { id: 'modern', icon: '🌆', label: 'Modern' },
          ].map((template) => (
            <button
              key={template.id}
              onClick={() => onCreateNew?.()}
              className="px-3 py-1.5 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg text-sm flex items-center gap-1.5"
            >
              <span>{template.icon}</span>
              <span>{template.label}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}