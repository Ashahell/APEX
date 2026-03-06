import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  filePaths: z.array(z.string()).describe('Files to generate documentation for'),
  format: z.enum(['markdown', 'html', 'mdbook', 'jsdoc']).describe('Documentation format'),
  title: z.string().optional().describe('Documentation title'),
});

const OutputSchema = z.object({
  docs: z.array(z.object({
    path: z.string(),
    content: z.string(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'code.document',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { filePaths, format, title } = input;
    
    try {
      const docs = await generateDocs(filePaths, format, title);
      
      return {
        success: true,
        output: JSON.stringify({
          docs,
          summary: `Generated ${format} documentation for ${filePaths.length} files`,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Documentation generation failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

async function generateDocs(filePaths: string[], format: string, title?: string): Promise<{ path: string; content: string }[]> {
  const docs: { path: string; content: string }[] = [];
  const docTitle = title || 'Documentation';
  const fs = await import('fs/promises');
  const path = await import('path');
  
  for (const filePath of filePaths) {
    try {
      const content = await fs.readFile(filePath, 'utf-8');
      const ext = path.extname(filePath);
      const baseName = path.basename(filePath, ext);
      
      let docContent = '';
      
      switch (format) {
        case 'markdown':
          docContent = generateMarkdownDoc(baseName, ext, content);
          break;
        case 'jsdoc':
          docContent = generateJsDoc(baseName, ext, content);
          break;
        case 'html':
          docContent = generateHtmlDoc(baseName, ext, content);
          break;
        default:
          docContent = generateMarkdownDoc(baseName, ext, content);
      }
      
      docs.push({ 
        path: `${baseName}.${format === 'markdown' ? 'md' : format === 'jsdoc' ? 'js' : 'html'}`,
        content: docContent 
      });
    } catch (error) {
      docs.push({
        path: filePath,
        content: `# Error\nFailed to generate documentation: ${error}`,
      });
    }
  }
  
  return docs;
}

function generateMarkdownDoc(fileName: string, ext: string, content: string): string {
  const lines = content.split('\n');
  const functions: string[] = [];
  const classes: string[] = [];
  const imports: string[] = [];
  
  // Simple parsing for common patterns
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('function ') || trimmed.startsWith('const ') && trimmed.includes('=')) {
      const match = trimmed.match(/(?:function|const)\s+(\w+)/);
      if (match) functions.push(match[1]);
    }
    if (trimmed.startsWith('class ')) {
      const match = trimmed.match(/class\s+(\w+)/);
      if (match) classes.push(match[1]);
    }
    if (trimmed.startsWith('import ')) {
      imports.push(trimmed);
    }
  }
  
  let doc = `# ${fileName}\n\n`;
  doc += `## Overview\n\nAuto-generated documentation for \`${fileName}\`.\n\n`;
  
  if (imports.length > 0) {
    doc += `## Imports\n\n${imports.map(i => '- `' + i + '`').join('\n')}\n\n`;
  }
  
  if (classes.length > 0) {
    doc += `## Classes\n\n${classes.map(c => '- `' + c + '`').join('\n')}\n\n`;
  }
  
  if (functions.length > 0) {
    doc += `## Functions\n\n${functions.map(f => '- `' + f + '()`').join('\n')}\n\n`;
  }
  
  if (functions.length === 0 && classes.length === 0 && imports.length === 0) {
    doc += `## Content\n\n\`\`\`${ext.slice(1)}\n${content.slice(0, 1000)}\n\`\`\`\n\n`;
  }
  
  return doc;
}

function generateJsDoc(fileName: string, ext: string, content: string): string {
  const lines = content.split('\n');
  let doc = `/**\n * ${fileName} module\n * @module ${fileName}\n */\n\n`;
  
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('function ') || trimmed.match(/^(?:const|let|var)\s+\w+\s*=/)) {
      const match = trimmed.match(/(?:function|const|let|var)\s+(\w+)/);
      if (match) {
        doc += `/**\n * @function ${match[1]}\n * @description Auto-generated function documentation\n */\n`;
      }
    }
  }
  
  return doc;
}

function generateHtmlDoc(fileName: string, ext: string, content: string): string {
  return `<!DOCTYPE html>
<html>
<head>
  <title>${fileName} - Documentation</title>
  <style>
    body { font-family: system-ui; max-width: 800px; margin: 0 auto; padding: 20px; }
    pre { background: #f4f4f4; padding: 10px; overflow-x: auto; }
  </style>
</head>
<body>
  <h1>${fileName}</h1>
  <pre><code>${content.slice(0, 2000)}${content.length > 2000 ? '...' : ''}</code></pre>
</body>
</html>`;
}
  
  return docs;
}
   
export default skill;
