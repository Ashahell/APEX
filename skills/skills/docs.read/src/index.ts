import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  path: z.string().describe('Path to documentation file or URL'),
  query: z.string().optional().describe('Specific information to find'),
  maxLength: z.number().optional().default(5000).describe('Max characters to read'),
});

const OutputSchema = z.object({
  content: z.string(),
  summary: z.string(),
  sections: z.array(z.string()),
});

export const skill: Skill = {
  name: 'docs.read',
  version: '0.1.0',
  tier: 'T0',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { path, query, maxLength } = input;
    
    try {
      let content: string;
      
      if (path.startsWith('http://') || path.startsWith('https://')) {
        content = await fetchRemoteDoc(path);
      } else {
        content = await readLocalDoc(path);
      }
      
      if (query) {
        content = filterByQuery(content, query);
      }
      
      if (content.length > maxLength) {
        content = content.substring(0, maxLength) + '\n\n[Truncated...]';
      }
      
      const sections = extractSections(content);
      const summary = generateSummary(content, query);
      
      return {
        success: true,
        output: JSON.stringify({
          content,
          summary,
          sections,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Failed to read documentation: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

async function fetchRemoteDoc(url: string): Promise<string> {
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return await response.text();
  } catch (error) {
    throw new Error(`Failed to fetch URL: ${error}`);
  }
}

async function readLocalDoc(path: string): Promise<string> {
  try {
    const { readFile } = await import('fs/promises');
    return await readFile(path, 'utf-8');
  } catch (error) {
    throw new Error(`Failed to read file: ${error}`);
  }
}

function filterByQuery(content: string, query: string): string {
  const queryLower = query.toLowerCase();
  const lines = content.split('\n');
  const relevantLines: string[] = [];
  
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].toLowerCase().includes(queryLower)) {
      const start = Math.max(0, i - 2);
      const end = Math.min(lines.length, i + 3);
      relevantLines.push(...lines.slice(start, end));
      relevantLines.push('---');
    }
  }
  
  if (relevantLines.length === 0) {
    return content;
  }
  
  return relevantLines.join('\n');
}

function extractSections(content: string): string[] {
  const sections: string[] = [];
  const lines = content.split('\n');
  
  for (const line of lines) {
    const match = line.match(/^(#{1,6})\s+(.+)$/);
    if (match) {
      sections.push(match[2]);
    }
  }
  
  return sections;
}

function generateSummary(content: string, query?: string): string {
  const lines = content.split('\n').filter(l => l.trim());
  
  if (query) {
    return `Found ${lines.length} lines matching "${query}".`;
  }
  
  const wordCount = content.split(/\s+/).length;
  const charCount = content.length;
  
  return `Document contains ${wordCount} words (${charCount} characters) across ${lines.length} lines.`;
}

export default skill;
