import { useState, useEffect } from 'react';
import { 
  listSlackTemplates, 
  createSlackTemplate, 
  updateSlackTemplate, 
  deleteSlackTemplate,
  renderSlackTemplate,
  type SlackBlockTemplate 
} from '../../lib/api';

export function SlackBlockManager() {
  const [templates, setTemplates] = useState<SlackBlockTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedTemplate, setSelectedTemplate] = useState<SlackBlockTemplate | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [renderedPreview, setRenderedPreview] = useState('');
  const [renderVariables, setRenderVariables] = useState('{}');
  
  // Form state
  const [formName, setFormName] = useState('');
  const [formTemplate, setFormTemplate] = useState('');
  const [formDescription, setFormDescription] = useState('');

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      const data = await listSlackTemplates();
      setTemplates(data);
    } catch (err) {
      console.error('Failed to load templates:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = (template: SlackBlockTemplate) => {
    setSelectedTemplate(template);
    setFormName(template.name);
    setFormTemplate(template.template);
    setFormDescription(template.description || '');
    setIsEditing(false);
    setRenderedPreview('');
  };

  const handleCreate = () => {
    setSelectedTemplate(null);
    setFormName('');
    setFormTemplate('');
    setFormDescription('');
    setIsEditing(true);
    setRenderedPreview('');
  };

  const handleSave = async () => {
    try {
      if (selectedTemplate) {
        await updateSlackTemplate(selectedTemplate.id, {
          name: formName,
          template: formTemplate,
          description: formDescription,
        });
      } else {
        await createSlackTemplate({
          name: formName,
          template: formTemplate,
          description: formDescription,
        });
      }
      await loadTemplates();
      setIsEditing(false);
    } catch (err) {
      console.error('Failed to save template:', err);
    }
  };

  const handleDelete = async () => {
    if (!selectedTemplate) return;
    if (!confirm('Are you sure you want to delete this template?')) return;
    
    try {
      await deleteSlackTemplate(selectedTemplate.id);
      setSelectedTemplate(null);
      await loadTemplates();
    } catch (err) {
      console.error('Failed to delete template:', err);
    }
  };

  const handlePreview = async () => {
    if (!selectedTemplate) return;
    
    try {
      let variables;
      try {
        variables = JSON.parse(renderVariables);
      } catch {
        variables = {};
      }
      
      const result = await renderSlackTemplate(selectedTemplate.id, variables);
      setRenderedPreview(result.rendered);
    } catch (err) {
      console.error('Failed to render preview:', err);
      setRenderedPreview('Error rendering template');
    }
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
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Slack Block Kit Templates</h3>
          <p className="text-sm text-muted-foreground">
            Manage rich Slack message templates with Block Kit
          </p>
        </div>
        <button
          onClick={handleCreate}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90"
        >
          + New Template
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Template List */}
        <div className="border rounded-lg p-4">
          <h4 className="font-medium mb-4">Templates</h4>
          <div className="space-y-2">
            {templates.map(template => (
              <button
                key={template.id}
                onClick={() => handleSelect(template)}
                className={`w-full text-left p-3 rounded-lg border transition-colors ${
                  selectedTemplate?.id === template.id
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <div className="font-medium">{template.name}</div>
                {template.description && (
                  <div className="text-sm text-muted-foreground truncate">
                    {template.description}
                  </div>
                )}
              </button>
            ))}
            {templates.length === 0 && (
              <div className="text-center py-8 text-muted-foreground">
                No templates yet
              </div>
            )}
          </div>
        </div>

        {/* Template Editor / Preview */}
        <div className="border rounded-lg p-4">
          {(isEditing || selectedTemplate) ? (
            <>
              <div className="flex items-center justify-between mb-4">
                <h4 className="font-medium">
                  {selectedTemplate ? 'Edit Template' : 'New Template'}
                </h4>
                <div className="flex gap-2">
                  {selectedTemplate && (
                    <>
                      <button
                        onClick={() => setIsEditing(!isEditing)}
                        className="px-3 py-1 text-sm border rounded-md hover:bg-muted"
                      >
                        {isEditing ? 'Cancel' : 'Edit'}
                      </button>
                      <button
                        onClick={handleDelete}
                        className="px-3 py-1 text-sm text-red-500 border border-red-500 rounded-md hover:bg-red-50"
                      >
                        Delete
                      </button>
                    </>
                  )}
                </div>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-1">Name</label>
                  <input
                    type="text"
                    value={formName}
                    onChange={(e) => setFormName(e.target.value)}
                    disabled={!isEditing && !!selectedTemplate}
                    className="w-full px-3 py-2 bg-background border rounded-md text-sm"
                    placeholder="e.g., task_complete"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium mb-1">Description</label>
                  <input
                    type="text"
                    value={formDescription}
                    onChange={(e) => setFormDescription(e.target.value)}
                    disabled={!isEditing && !!selectedTemplate}
                    className="w-full px-3 py-2 bg-background border rounded-md text-sm"
                    placeholder="Template description"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium mb-1">
                    Template (JSON)
                    <span className="text-muted-foreground font-normal ml-2">
                      Use {'{{variable}}'} syntax
                    </span>
                  </label>
                  <textarea
                    value={formTemplate}
                    onChange={(e) => setFormTemplate(e.target.value)}
                    disabled={!isEditing && !!selectedTemplate}
                    className="w-full px-3 py-2 bg-background border rounded-md text-sm font-mono"
                    rows={8}
                    placeholder='{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "Hello {{name}}"}}]}'
                  />
                </div>

                {(isEditing || !selectedTemplate) && (
                  <button
                    onClick={handleSave}
                    disabled={!formName || !formTemplate}
                    className="w-full px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
                  >
                    Save Template
                  </button>
                )}

                {selectedTemplate && !isEditing && (
                  <div className="border-t pt-4 mt-4">
                    <h5 className="font-medium mb-2">Preview</h5>
                    <div className="flex gap-2 mb-2">
                      <input
                        type="text"
                        value={renderVariables}
                        onChange={(e) => setRenderVariables(e.target.value)}
                        className="flex-1 px-3 py-2 bg-background border rounded-md text-sm font-mono"
                        placeholder='{"name": "World"}'
                      />
                      <button
                        onClick={handlePreview}
                        className="px-3 py-2 bg-muted border rounded-md hover:bg-muted/80 text-sm"
                      >
                        Render
                      </button>
                    </div>
                    {renderedPreview && (
                      <pre className="p-3 bg-muted rounded-md text-xs font-mono overflow-auto max-h-48">
                        {renderedPreview}
                      </pre>
                    )}
                  </div>
                )}
              </div>
            </>
          ) : (
            <div className="text-center py-12 text-muted-foreground">
              Select a template to view or edit
            </div>
          )}
        </div>
      </div>

      {/* Info Section */}
      <div className="bg-muted/50 p-4 rounded-lg">
        <h4 className="font-medium mb-2">About Slack Block Kit</h4>
        <p className="text-sm text-muted-foreground">
          Slack Block Kit allows you to create rich, interactive messages. 
          Use {'{{variable}}'} syntax in your templates to inject dynamic data.
          Common variables: task_name, summary, error_message, task_id, budget_limit, current_cost, percentage.
        </p>
      </div>
    </div>
  );
}
