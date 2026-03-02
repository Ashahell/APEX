import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  code: z.string().describe('Code to review'),
  language: z.string().describe('Programming language'),
  focus: z.enum(['bugs', 'style', 'security', 'performance', 'all']).optional().describe('Review focus area'),
});

const OutputSchema = z.object({
  issues: z.array(z.object({
    severity: z.enum(['error', 'warning', 'info']),
    line: z.number().optional(),
    message: z.string(),
    suggestion: z.string().optional(),
  })),
  summary: z.string(),
  score: z.number().min(0).max(10),
});

export const skill: Skill = {
  name: 'code.review',
  version: '0.1.0',
  tier: 'T0',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { code, language, focus } = input;
    
    try {
      const issues = performReview(code, language, focus || 'all');
      const score = calculateScore(issues);
      const summary = generateSummary(issues, score);
      
      return {
        success: true,
        output: JSON.stringify({
          issues,
          summary,
          score,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Code review failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function performReview(
  code: string,
  language: string,
  focus: string
): Array<{ severity: 'error' | 'warning' | 'info'; line?: number; message: string; suggestion?: string }> {
  const issues: Array<{ severity: 'error' | 'warning' | 'info'; line?: number; message: string; suggestion?: string }> = [];
  const lines = code.split('\n');
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const lineNum = i + 1;
    
    if (focus === 'all' || focus === 'bugs') {
      if (line.includes('==') && !line.includes('===')) {
        issues.push({
          severity: 'warning',
          line: lineNum,
          message: 'Use === instead of ==',
          suggestion: 'Replace == with === for strict equality',
        });
      }
      
      if (line.includes('eval(')) {
        issues.push({
          severity: 'error',
          line: lineNum,
          message: 'Use of eval() is dangerous',
          suggestion: 'Avoid eval(), use JSON.parse() or other safer alternatives',
        });
      }
    }
    
    if (focus === 'all' || focus === 'style') {
      if (line.length > 120) {
        issues.push({
          severity: 'info',
          line: lineNum,
          message: 'Line exceeds 120 characters',
          suggestion: 'Consider breaking long lines for readability',
        });
      }
      
      if (line.trimEnd() !== line) {
        issues.push({
          severity: 'info',
          line: lineNum,
          message: 'Trailing whitespace',
          suggestion: 'Remove trailing whitespace',
        });
      }
    }
    
    if (focus === 'all' || focus === 'security') {
      if (line.includes('password') && line.includes('=') && !line.includes('encrypted')) {
        issues.push({
          severity: 'warning',
          line: lineNum,
          message: 'Potential hardcoded password',
          suggestion: 'Use environment variables for sensitive data',
        });
      }
    }
  }
  
  if (code.length === 0) {
    issues.push({
      severity: 'error',
      message: 'Empty code provided',
    });
  }
  
  return issues;
}

function calculateScore(
  issues: Array<{ severity: 'error' | 'warning' | 'info' }>
): number {
  let score = 10;
  
  for (const issue of issues) {
    if (issue.severity === 'error') score -= 2;
    else if (issue.severity === 'warning') score -= 0.5;
    else if (issue.severity === 'info') score -= 0.1;
  }
  
  return Math.max(0, Math.round(score * 10) / 10);
}

function generateSummary(
  issues: Array<{ severity: string }>,
  score: number
): string {
  const errors = issues.filter(i => i.severity === 'error').length;
  const warnings = issues.filter(i => i.severity === 'warning').length;
  const infos = issues.filter(i => i.severity === 'info').length;
  
  return `Code score: ${score}/10. Found ${errors} error(s), ${warnings} warning(s), ${infos} suggestion(s).`;
}

export default skill;
