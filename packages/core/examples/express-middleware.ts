/**
 * Express.js Middleware Example
 *
 * Demonstrates how to integrate LLM Shield with Express.js
 */

// Note: This example requires installing express
// npm install express @types/express

import { LLMShield } from '@llm-shield/core';

// Mock Express types for this example
interface Request {
  body: Record<string, unknown>;
}

interface Response {
  status(code: number): Response;
  json(data: unknown): Response;
}

interface NextFunction {
  (): void;
}

// Create a singleton shield instance
const shield = new LLMShield({
  cache: { maxSize: 10000, ttlSeconds: 3600 },
  scanners: ['prompt-injection', 'toxicity', 'secrets'],
  debug: false,
});

/**
 * Middleware to scan incoming prompts
 */
export function scanPromptMiddleware(
  req: Request,
  res: Response,
  next: NextFunction
) {
  const { prompt } = req.body;

  if (!prompt) {
    return res.status(400).json({
      error: 'Missing prompt in request body',
    });
  }

  shield
    .scanPrompt(prompt as string)
    .then((result) => {
      if (!result.isValid) {
        return res.status(400).json({
          error: 'Prompt rejected by security scan',
          reason: result.detections.map((d) => d.description),
          riskScore: result.riskScore,
        });
      }

      // Prompt is safe, continue to next middleware
      next();
    })
    .catch((error) => {
      console.error('Scan error:', error);
      return res.status(500).json({
        error: 'Internal security scan error',
      });
    });
}

/**
 * Middleware to scan LLM outputs
 */
export function scanOutputMiddleware(output: string): Promise<boolean> {
  return shield
    .scanOutput(output)
    .then((result) => {
      if (!result.isValid) {
        console.warn('Output rejected:', result.detections);
        return false;
      }
      return true;
    });
}

// Example Express app setup
export function createApp() {
  const express = require('express');
  const app = express();

  app.use(express.json());

  // Apply scan middleware to chat endpoint
  app.post('/api/chat', scanPromptMiddleware, async (req: Request, res: Response) => {
    try {
      // Your LLM integration here
      const response = await callLLM(req.body.prompt);

      // Scan the output
      const isOutputSafe = await scanOutputMiddleware(response);

      if (!isOutputSafe) {
        return res.status(500).json({
          error: 'Response rejected by security scan',
        });
      }

      return res.json({ response });
    } catch (error) {
      return res.status(500).json({
        error: 'Internal server error',
      });
    }
  });

  // Health check endpoint
  app.get('/health', (req: Request, res: Response) => {
    const stats = shield.getCacheStats();
    res.json({
      status: 'healthy',
      cache: stats,
    });
  });

  return app;
}

// Mock LLM function
async function callLLM(prompt: unknown): Promise<string> {
  return `Response to: ${prompt}`;
}

// Example usage
if (require.main === module) {
  const app = createApp();
  const port = process.env.PORT || 3000;

  app.listen(port, () => {
    console.log(`Server running on port ${port}`);
  });
}
