import { useEffect, useState } from 'react';

interface Skill {
  name: string;
  version: string;
  tier: string;
  status: string;
}

export function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch('http://localhost:3000/api/v1/skills')
      .then((res) => res.json())
      .then((data) => {
        setSkills(data);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message);
        setLoading(false);
      });
  }, []);

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'T0':
        return 'bg-green-100 text-green-800';
      case 'T1':
        return 'bg-blue-100 text-blue-800';
      case 'T2':
        return 'bg-yellow-100 text-yellow-800';
      case 'T3':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading skills...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 flex items-center justify-center h-full">
        <div className="text-red-500">Error: {error}</div>
      </div>
    );
  }

  return (
    <div className="p-4 h-full overflow-y-auto">
      <div className="mb-4">
        <h2 className="text-2xl font-semibold">Skills Marketplace</h2>
        <p className="text-muted-foreground">Browse and manage available skills</p>
      </div>

      {skills.length === 0 ? (
        <div className="text-center py-8">
          <p className="text-muted-foreground">No skills registered</p>
          <p className="text-sm text-muted-foreground mt-2">
            Skills can be registered via the API
          </p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {skills.map((skill) => (
            <div
              key={skill.name}
              className="border rounded-lg p-4 hover:shadow-md transition-shadow"
            >
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-semibold">{skill.name}</h3>
                <span
                  className={`px-2 py-1 rounded text-xs font-medium ${getTierColor(
                    skill.tier
                  )}`}
                >
                  {skill.tier}
                </span>
              </div>
              <div className="text-sm text-muted-foreground">
                <p>Version: {skill.version}</p>
                <p>Status: {skill.status || 'active'}</p>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="mt-8 border-t pt-4">
        <h3 className="font-semibold mb-2">Quick Stats</h3>
        <div className="grid grid-cols-3 gap-4">
          <div className="bg-muted rounded-lg p-4 text-center">
            <div className="text-2xl font-bold">{skills.length}</div>
            <div className="text-xs text-muted-foreground">Total Skills</div>
          </div>
          <div className="bg-muted rounded-lg p-4 text-center">
            <div className="text-2xl font-bold">
              {skills.filter((s) => s.tier === 'T0').length}
            </div>
            <div className="text-xs text-muted-foreground">T0 Skills</div>
          </div>
          <div className="bg-muted rounded-lg p-4 text-center">
            <div className="text-2xl font-bold">
              {skills.filter((s) => s.tier === 'T1').length}
            </div>
            <div className="text-xs text-muted-foreground">T1 Skills</div>
          </div>
        </div>
      </div>
    </div>
  );
}
