import { useState, useEffect } from 'react';
import { listPins, unpinMessage, PinnedMessage } from '../../lib/dashboard';

export function PinnedMessages() {
  const [pins, setPins] = useState<PinnedMessage[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPins();
  }, []);

  const loadPins = async () => {
    try {
      const data = await listPins();
      setPins(data);
    } catch (err) {
      console.error('Failed to load pins:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleUnpin = async (id: string) => {
    try {
      await unpinMessage(id);
      setPins(pins.filter((p) => p.id !== id));
    } catch (err) {
      console.error('Failed to unpin:', err);
    }
  };

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center">
        <div className="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (pins.length === 0) {
    return (
      <div className="p-4 text-center text-gray-500 text-sm">
        No pinned messages
      </div>
    );
  }

  return (
    <ul className="overflow-y-auto max-h-64">
      {pins.map((pin) => (
        <li key={pin.id} className="border-b border-gray-700/50 last:border-0">
          <div className="p-3 hover:bg-gray-800/50 group">
            <div className="flex items-start justify-between gap-2">
              <div className="flex-1 min-w-0">
                <p className="text-sm text-white truncate">{pin.message_id}</p>
                <p className="text-xs text-gray-500 mt-1">
                  {new Date(pin.pinned_at).toLocaleDateString()}
                </p>
                {pin.pin_note && (
                  <p className="text-xs text-gray-400 mt-1 italic">{pin.pin_note}</p>
                )}
              </div>
              <button
                onClick={() => handleUnpin(pin.id)}
                className="opacity-0 group-hover:opacity-100 text-gray-500 hover:text-red-400 transition-opacity"
                title="Unpin"
              >
                ×
              </button>
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}
