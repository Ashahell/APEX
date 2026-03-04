import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  product: z.string().describe('Product or service name'),
  audience: z.string().describe('Target audience'),
  tone: z.enum(['professional', 'friendly', 'urgent', 'humorous']).describe('Copy tone'),
  channels: z.array(z.enum(['website', 'email', 'social', 'ads'])).describe('Distribution channels'),
  keyBenefits: z.array(z.string()).describe('Key benefits to highlight'),
});

const OutputSchema = z.object({
  copies: z.record(z.string()),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'copy.generate',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { product, audience, tone, channels, keyBenefits } = input;
    
    try {
      const copies: Record<string, string> = {};
      
      if (channels.includes('website')) {
        copies.website = generateWebsiteCopy(product, audience, tone, keyBenefits);
      }
      if (channels.includes('email')) {
        copies.email = generateEmailCopy(product, audience, tone, keyBenefits);
      }
      if (channels.includes('social')) {
        copies.social = generateSocialCopy(product, audience, tone, keyBenefits);
      }
      if (channels.includes('ads')) {
        copies.ads = generateAdsCopy(product, audience, tone, keyBenefits);
      }
      
      return {
        success: true,
        output: JSON.stringify({
          copies,
          summary: 'Generated ' + Object.keys(copies).length + ' copies for: ' + product,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Copy generation failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function generateWebsiteCopy(product: string, audience: string, tone: string, benefits: string[]): string {
  return 'Transform your ' + audience + ' experience with ' + product + '. ' + benefits.join('. ') + '.';
}

function generateEmailCopy(product: string, audience: string, tone: string, benefits: string[]): string {
  return 'Subject: Discover ' + product + '\n\nHi ' + audience + ',\n\n' + benefits[0] + '.';
}

function generateSocialCopy(product: string, audience: string, tone: string, benefits: string[]): string {
  return product + ': ' + benefits[0] + ' #' + audience.replace(/\s/g, '');
}

function generateAdsCopy(product: string, audience: string, tone: string, benefits: string[]): string {
  return product + ' - ' + benefits[0] + ' - Learn more!';
}

export default skill;
