import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../../src/types.js';

const InputSchema = z.object({
  action: z.enum(['invoke', 'deploy', 'delete', 'list', 'get-config'])
    .describe('AWS Lambda action to perform'),
  functionName: z.string().describe('Lambda function name'),
  region: z.string().optional().default('us-east-1').describe('AWS region'),
  payload: z.string().optional().describe('JSON payload for invocation'),
  runtime: z.string().optional().default('nodejs20x').describe('Lambda runtime'),
  handler: z.string().optional().default('index.handler').describe('Handler function'),
  zipFile: z.string().optional().describe('Path to zip file for deployment'),
  memorySize: z.number().optional().default(128).describe('Memory in MB'),
  timeout: z.number().optional().default(3).describe('Timeout in seconds'),
});

const OutputSchema = z.object({
  success: z.boolean(),
  action: z.string(),
  functionName: z.string(),
  result: z.string().optional(),
  error: z.string().optional(),
});

export const skill: Skill = {
  name: 'aws.lambda',
  version: '0.1.0',
  tier: 'T3',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    if (ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T3 permission required for AWS Lambda operations',
      };
    }

    const { action, functionName, region, payload, runtime, handler, zipFile, memorySize, timeout } = input;

    try {
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);

      let command = '';
      let result = '';

      switch (action) {
        case 'invoke':
          command = `aws lambda invoke --function-name ${functionName} --payload '${payload || '{}'}' --region ${region} /tmp/lambda-response.json`;
          await execAsync(command);
          const fs = await import('fs/promises');
          result = await fs.readFile('/tmp/lambda-response.json', 'utf-8');
          break;

        case 'deploy':
          if (!zipFile) {
            return { success: false, error: 'zipFile required for deploy action' };
          }
          command = `aws lambda create-function --function-name ${functionName} --runtime ${runtime} --role arn:aws:iam::123456789012:role/lambda-ex --handler ${handler} --zip-file fileb://${zipFile} --memory-size ${memorySize} --timeout ${timeout} --region ${region}`;
          await execAsync(command).catch((err) => {
            if (err.message.includes('Function already exist')) {
              command = `aws lambda update-function-code --function-name ${functionName} --zip-file fileb://${zipFile} --region ${region}`;
              return execAsync(command);
            }
            throw err;
          });
          result = `Lambda function ${functionName} deployed successfully`;
          break;

        case 'delete':
          command = `aws lambda delete-function --function-name ${functionName} --region ${region}`;
          await execAsync(command);
          result = `Lambda function ${functionName} deleted successfully`;
          break;

        case 'list':
          command = `aws lambda list-functions --region ${region} --max-items 10`;
          result = (await execAsync(command)).stdout;
          break;

        case 'get-config':
          command = `aws lambda get-function-configuration --function-name ${functionName} --region ${region}`;
          result = (await execAsync(command)).stdout;
          break;
      }

      return {
        success: true,
        output: JSON.stringify({
          success: true,
          action,
          functionName,
          result,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `AWS Lambda operation failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    try {
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);
      await execAsync('aws --version');
      return true;
    } catch {
      return false;
    }
  },
};

export default skill;
