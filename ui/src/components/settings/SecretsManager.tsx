import { useState, useEffect } from 'react';
import { 
  listSecrets, 
  listSecretCategories, 
  getSecretsByCategory,
  type SecretRef 
} from '../../lib/api';

export function SecretsManager() {
  const [secrets, setSecrets] = useState<SecretRef[]>([]);
  const [categories, setCategories] = useState<string[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [secretsData, categoriesData] = await Promise.all([
        listSecrets(),
        listSecretCategories()
      ]);
      setSecrets(secretsData);
      setCategories(categoriesData);
    } catch (err) {
      console.error('Failed to load secrets:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleCategoryChange = async (category: string) => {
    setSelectedCategory(category);
    setLoading(true);
    try {
      if (category === 'all') {
        const data = await listSecrets();
        setSecrets(data);
      } else {
        const data = await getSecretsByCategory(category);
        setSecrets(data);
      }
    } catch (err) {
      console.error('Failed to load secrets:', err);
    } finally {
      setLoading(false);
    }
  };

  const filteredSecrets = secrets.filter(s => 
    s.secret_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    s.ref_key.toLowerCase().includes(searchQuery.toLowerCase()) ||
    (s.description?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false)
  );

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'api_key': return '🔑';
      case 'token': return '🎫';
      case 'credential': return '🔐';
      case 'certificate': return '📜';
      case 'generic': return '⚙️';
      default: return '❓';
    }
  };

  const getStatusBadge = (isPredefined: boolean) => {
    if (isPredefined) {
      return <span className="px-2 py-0.5 text-xs bg-indigo-500/20 text-indigo-400 rounded">Predefined</span>;
    }
    return <span className="px-2 py-0.5 text-xs bg-green-500/20 text-green-400 rounded">Custom</span>;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">Secrets Management</h3>
        <p className="text-sm text-muted-foreground">
          Manage secret references and credentials. These map to environment variables for tools and adapters.
        </p>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap gap-4 items-center">
        <div className="flex-1 min-w-[200px]">
          <input
            type="text"
            placeholder="Search secrets..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-3 py-2 bg-background border rounded-md text-sm"
          />
        </div>
        
        <select
          value={selectedCategory}
          onChange={(e) => handleCategoryChange(e.target.value)}
          className="px-3 py-2 bg-background border rounded-md text-sm"
        >
          <option value="all">All Categories</option>
          {categories.map(cat => (
            <option key={cat} value={cat}>
              {getCategoryIcon(cat)} {cat.charAt(0).toUpperCase() + cat.slice(1)}
            </option>
          ))}
        </select>
      </div>

      {/* Secrets Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {filteredSecrets.map(secret => (
          <div 
            key={secret.id} 
            className="p-4 border rounded-lg bg-card hover:border-primary/50 transition-colors"
          >
            <div className="flex items-start justify-between mb-2">
              <div className="flex items-center gap-2">
                <span className="text-lg">{getCategoryIcon(secret.category)}</span>
                <h4 className="font-medium">{secret.secret_name}</h4>
              </div>
              {getStatusBadge(secret.is_predefined)}
            </div>
            
            <div className="space-y-1 text-sm text-muted-foreground">
              <div className="flex items-center gap-2">
                <span className="text-xs uppercase tracking-wider">Ref:</span>
                <code className="px-1.5 py-0.5 bg-muted rounded text-xs">{secret.ref_key}</code>
              </div>
              
              {secret.env_var && (
                <div className="flex items-center gap-2">
                  <span className="text-xs uppercase tracking-wider">Env:</span>
                  <code className="px-1.5 py-0.5 bg-muted rounded text-xs">{secret.env_var}</code>
                </div>
              )}
              
              {secret.description && (
                <p className="text-xs mt-2">{secret.description}</p>
              )}
              
              {secret.targets.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-2">
                  {secret.targets.slice(0, 3).map((target, idx) => (
                    <span 
                      key={idx} 
                      className="px-1.5 py-0.5 text-xs bg-muted/50 rounded"
                    >
                      {target}
                    </span>
                  ))}
                  {secret.targets.length > 3 && (
                    <span className="px-1.5 py-0.5 text-xs text-muted-foreground">
                      +{secret.targets.length - 3} more
                    </span>
                  )}
                </div>
              )}
            </div>
          </div>
        ))}
      </div>

      {filteredSecrets.length === 0 && (
        <div className="text-center py-8 text-muted-foreground">
          No secrets found matching your search.
        </div>
      )}

      {/* Info Section */}
      <div className="bg-muted/50 p-4 rounded-lg">
        <h4 className="font-medium mb-2">About Secrets</h4>
        <p className="text-sm text-muted-foreground">
          Predefined secrets are built-in references for common credentials (API keys, tokens, etc.). 
          Custom secrets (CUSTOM_SECRET_1-5) can be used for your own integrations. 
          Secret values should be stored securely via environment variables or the encrypted Preferences storage.
        </p>
      </div>
    </div>
  );
}
