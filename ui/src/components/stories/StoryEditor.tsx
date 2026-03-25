import { useState } from 'react';

interface StoryCharacter {
  name: string;
  description: string;
  hp?: number;
}

interface StoryEditorProps {
  onCreate?: (story: { title: string; setting: string; characters: StoryCharacter[] }) => void;
  onCancel?: () => void;
}

const SETTINGS = [
  { id: 'fantasy', name: 'Fantasy', icon: '⚔️', description: 'Knights, dragons, and magic' },
  { id: 'scifi', name: 'Sci-Fi', icon: '🚀', description: 'Space exploration and technology' },
  { id: 'horror', name: 'Horror', icon: '👻', description: 'Spooky encounters and survival' },
  { id: 'mystery', name: 'Mystery', icon: '🔍', description: 'Puzzles and investigation' },
  { id: 'western', name: 'Western', icon: '🤠', description: 'Cowboys and frontier adventures' },
  { id: 'modern', name: 'Modern', icon: '🌆', description: 'Contemporary settings and challenges' },
];

export function StoryEditor({ onCreate, onCancel }: StoryEditorProps) {
  const [title, setTitle] = useState('');
  const [setting, setSetting] = useState('fantasy');
  const [characters, setCharacters] = useState<StoryCharacter[]>([
    { name: '', description: '' }
  ]);
  const [initialPrompt, setInitialPrompt] = useState('');

  const handleAddCharacter = () => {
    setCharacters([...characters, { name: '', description: '' }]);
  };

  const handleRemoveCharacter = (index: number) => {
    if (characters.length > 1) {
      setCharacters(characters.filter((_, i) => i !== index));
    }
  };

  const handleCharacterChange = (index: number, field: keyof StoryCharacter, value: string | number) => {
    const updated = [...characters];
    updated[index] = { ...updated[index], [field]: value };
    setCharacters(updated);
  };

  const handleSubmit = () => {
    const validCharacters = characters.filter(c => c.name.trim());
    onCreate?.({
      title: title || 'Untitled Adventure',
      setting,
      characters: validCharacters,
    });
  };

  return (
    <div className="max-w-2xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100">
          Create New Story
        </h2>
        <button
          onClick={onCancel}
          className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
        >
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {/* Title */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Story Title
        </label>
        <input
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="The Dragon's Lair"
          className="w-full px-4 py-2 border rounded-lg dark:bg-gray-800 dark:border-gray-700 focus:ring-indigo-500 focus:border-indigo-500"
        />
      </div>

      {/* Setting */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Setting / Genre
        </label>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
          {SETTINGS.map(s => (
            <button
              key={s.id}
              onClick={() => setSetting(s.id)}
              className={`p-3 rounded-lg border text-left transition-all ${
                setting === s.id
                  ? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/30'
                  : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
              }`}
            >
              <div className="text-2xl mb-1">{s.icon}</div>
              <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                {s.name}
              </div>
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {s.description}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Characters */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Characters
          </label>
          <button
            onClick={handleAddCharacter}
            className="text-xs text-indigo-600 hover:text-indigo-700 dark:text-indigo-400"
          >
            + Add Character
          </button>
        </div>
        <div className="space-y-3">
          {characters.map((char, idx) => (
            <div
              key={idx}
              className="flex gap-3 p-3 bg-gray-50 dark:bg-gray-800 rounded-lg border"
            >
              <div className="flex-1 space-y-2">
                <input
                  type="text"
                  value={char.name}
                  onChange={(e) => handleCharacterChange(idx, 'name', e.target.value)}
                  placeholder="Character name"
                  className="w-full px-3 py-1.5 text-sm border rounded dark:bg-gray-900 dark:border-gray-700"
                />
                <input
                  type="text"
                  value={char.description}
                  onChange={(e) => handleCharacterChange(idx, 'description', e.target.value)}
                  placeholder="Role or description"
                  className="w-full px-3 py-1.5 text-sm border rounded dark:bg-gray-900 dark:border-gray-700"
                />
              </div>
              {characters.length > 1 && (
                <button
                  onClick={() => handleRemoveCharacter(idx)}
                  className="text-gray-400 hover:text-red-500"
                >
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Initial Prompt */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
          Opening Scenario (Optional)
        </label>
        <textarea
          value={initialPrompt}
          onChange={(e) => setInitialPrompt(e.target.value)}
          placeholder="You stand at the entrance of a dark cave. The sound of dripping water echoes in the distance..."
          rows={4}
          className="w-full px-4 py-2 border rounded-lg dark:bg-gray-800 dark:border-gray-700 focus:ring-indigo-500 focus:border-indigo-500"
        />
        <p className="text-xs text-gray-400 mt-1">
          Describe the opening scene for your story. The AI will continue from here.
        </p>
      </div>

      {/* Actions */}
      <div className="flex justify-end gap-3 pt-4 border-t">
        <button
          onClick={onCancel}
          className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg"
        >
          Cancel
        </button>
        <button
          onClick={handleSubmit}
          className="px-6 py-2 bg-indigo-600 text-white font-medium rounded-lg hover:bg-indigo-700"
        >
          Start Adventure
        </button>
      </div>
    </div>
  );
}
