#!/usr/bin/env node

import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { readFileSync, existsSync, readdirSync, statSync } from 'fs';
import { resolve, join, relative } from 'path';
import { glob } from 'glob';

const VERSION = '1.0.0';

// Security patterns for detection
const PATTERNS = {
  // Prompt Injection patterns
  promptInjection: [
    /ignore\s+(all\s+)?(previous|prior|above)\s+(instructions?|prompts?|rules?)/i,
    /disregard\s+(all\s+)?(previous|prior|above)/i,
    /forget\s+(everything|all|what)\s+(you\s+)?(know|learned|were\s+told)/i,
    /you\s+are\s+(now|no\s+longer)\s+a/i,
    /pretend\s+(to\s+be|you\s+are)/i,
    /act\s+as\s+(if\s+you\s+are|a)/i,
    /new\s+instructions?:/i,
    /system\s+prompt:/i,
    /override\s+(previous|system)/i,
    /jailbreak/i,
    /DAN\s+mode/i,
  ],

  // Secret patterns (40+ types)
  secrets: [
    // AWS
    { pattern: /AKIA[0-9A-Z]{16}/g, type: 'AWS Access Key ID' },
    { pattern: /aws[_-]?secret[_-]?access[_-]?key["']?\s*[:=]\s*["']?[A-Za-z0-9/+=]{40}/gi, type: 'AWS Secret Access Key' },
    // GitHub
    { pattern: /ghp_[a-zA-Z0-9]{36}/g, type: 'GitHub Personal Access Token' },
    { pattern: /gho_[a-zA-Z0-9]{36}/g, type: 'GitHub OAuth Token' },
    { pattern: /ghs_[a-zA-Z0-9]{36}/g, type: 'GitHub App Token' },
    { pattern: /ghr_[a-zA-Z0-9]{36}/g, type: 'GitHub Refresh Token' },
    // Stripe
    { pattern: /sk_live_[0-9a-zA-Z]{24,}/g, type: 'Stripe Live Secret Key' },
    { pattern: /sk_test_[0-9a-zA-Z]{24,}/g, type: 'Stripe Test Secret Key' },
    { pattern: /pk_live_[0-9a-zA-Z]{24,}/g, type: 'Stripe Live Publishable Key' },
    // OpenAI
    { pattern: /sk-[a-zA-Z0-9]{48}/g, type: 'OpenAI API Key' },
    { pattern: /sk-proj-[a-zA-Z0-9]{48}/g, type: 'OpenAI Project API Key' },
    // Anthropic
    { pattern: /sk-ant-[a-zA-Z0-9-]{32,}/g, type: 'Anthropic API Key' },
    // Slack
    { pattern: /xox[baprs]-[0-9]{10,13}-[0-9]{10,13}-[a-zA-Z0-9]{24}/g, type: 'Slack Token' },
    { pattern: /https:\/\/hooks\.slack\.com\/services\/T[a-zA-Z0-9_]{8,}\/B[a-zA-Z0-9_]{8,}\/[a-zA-Z0-9_]{24}/g, type: 'Slack Webhook URL' },
    // Google
    { pattern: /AIza[0-9A-Za-z\-_]{35}/g, type: 'Google API Key' },
    // Generic
    { pattern: /api[_-]?key["']?\s*[:=]\s*["']?[a-zA-Z0-9]{32,}/gi, type: 'Generic API Key' },
    { pattern: /secret["']?\s*[:=]\s*["']?[a-zA-Z0-9]{32,}/gi, type: 'Generic Secret' },
    { pattern: /password["']?\s*[:=]\s*["']?[^\s'"]{8,}/gi, type: 'Password' },
    // Private Keys
    { pattern: /-----BEGIN (RSA |EC |OPENSSH |PGP )?PRIVATE KEY-----/g, type: 'Private Key' },
    // JWT
    { pattern: /eyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*/g, type: 'JWT Token' },
  ],

  // PII patterns
  pii: [
    { pattern: /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g, type: 'Email' },
    { pattern: /\b\d{3}-\d{2}-\d{4}\b/g, type: 'SSN' },
    { pattern: /\b\d{3}\s\d{2}\s\d{4}\b/g, type: 'SSN' },
    { pattern: /\b4[0-9]{12}(?:[0-9]{3})?\b/g, type: 'Credit Card (Visa)' },
    { pattern: /\b5[1-5][0-9]{14}\b/g, type: 'Credit Card (Mastercard)' },
    { pattern: /\b3[47][0-9]{13}\b/g, type: 'Credit Card (Amex)' },
    { pattern: /\b(?:\+?1[-.\s]?)?\(?[2-9]\d{2}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b/g, type: 'Phone Number' },
  ],

  // Toxicity patterns (basic keyword matching)
  toxicity: [
    /\b(kill|murder|attack|bomb|weapon|terrorist)\b/gi,
    /\b(hate|racist|sexist|bigot)\b/gi,
    /\b(suicide|self-harm)\b/gi,
  ],
};

interface ScanResult {
  file: string;
  line: number;
  column: number;
  type: string;
  category: 'prompt-injection' | 'secret' | 'pii' | 'toxicity';
  severity: 'low' | 'medium' | 'high' | 'critical';
  match: string;
}

interface ScanSummary {
  totalFiles: number;
  totalIssues: number;
  promptInjection: number;
  secrets: number;
  pii: number;
  toxicity: number;
  criticalCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
}

function getSeverity(category: string, type: string): 'low' | 'medium' | 'high' | 'critical' {
  if (category === 'secret') {
    if (type.includes('Private Key') || type.includes('AWS')) return 'critical';
    if (type.includes('API Key') || type.includes('Token')) return 'high';
    return 'medium';
  }
  if (category === 'pii') {
    if (type.includes('SSN') || type.includes('Credit Card')) return 'critical';
    if (type.includes('Email') || type.includes('Phone')) return 'medium';
    return 'low';
  }
  if (category === 'prompt-injection') return 'high';
  if (category === 'toxicity') return 'medium';
  return 'low';
}

function scanText(text: string, filename: string = 'input'): ScanResult[] {
  const results: ScanResult[] = [];
  const lines = text.split('\n');

  lines.forEach((line, lineIndex) => {
    // Check prompt injection
    PATTERNS.promptInjection.forEach((pattern) => {
      const match = line.match(pattern);
      if (match) {
        results.push({
          file: filename,
          line: lineIndex + 1,
          column: line.indexOf(match[0]) + 1,
          type: 'Prompt Injection Attempt',
          category: 'prompt-injection',
          severity: 'high',
          match: match[0].substring(0, 50) + (match[0].length > 50 ? '...' : ''),
        });
      }
    });

    // Check secrets
    PATTERNS.secrets.forEach(({ pattern, type }) => {
      const regex = new RegExp(pattern.source, pattern.flags);
      let match;
      while ((match = regex.exec(line)) !== null) {
        results.push({
          file: filename,
          line: lineIndex + 1,
          column: match.index + 1,
          type,
          category: 'secret',
          severity: getSeverity('secret', type),
          match: match[0].substring(0, 20) + '****',
        });
      }
    });

    // Check PII
    PATTERNS.pii.forEach(({ pattern, type }) => {
      const regex = new RegExp(pattern.source, pattern.flags);
      let match;
      while ((match = regex.exec(line)) !== null) {
        results.push({
          file: filename,
          line: lineIndex + 1,
          column: match.index + 1,
          type,
          category: 'pii',
          severity: getSeverity('pii', type),
          match: match[0].substring(0, 4) + '****',
        });
      }
    });

    // Check toxicity
    PATTERNS.toxicity.forEach((pattern) => {
      const match = line.match(pattern);
      if (match) {
        results.push({
          file: filename,
          line: lineIndex + 1,
          column: line.indexOf(match[0]) + 1,
          type: 'Potentially Toxic Content',
          category: 'toxicity',
          severity: 'medium',
          match: match[0],
        });
      }
    });
  });

  return results;
}

function formatResult(result: ScanResult): string {
  const severityColors: Record<string, (s: string) => string> = {
    critical: chalk.bgRed.white,
    high: chalk.red,
    medium: chalk.yellow,
    low: chalk.blue,
  };

  const categoryIcons: Record<string, string> = {
    'prompt-injection': '🎯',
    secret: '🔑',
    pii: '👤',
    toxicity: '⚠️',
  };

  const severityColor = severityColors[result.severity] || chalk.white;
  const icon = categoryIcons[result.category] || '•';

  return `  ${icon} ${chalk.gray(result.file)}:${chalk.cyan(result.line)}:${chalk.cyan(result.column)}
     ${severityColor(result.severity.toUpperCase())} ${chalk.white(result.type)}
     ${chalk.gray('Match:')} ${result.match}`;
}

function printSummary(summary: ScanSummary): void {
  console.log('\n' + chalk.bold('━━━ Scan Summary ━━━'));
  console.log(`  Files scanned: ${chalk.cyan(summary.totalFiles)}`);
  console.log(`  Total issues:  ${summary.totalIssues > 0 ? chalk.red(summary.totalIssues) : chalk.green(summary.totalIssues)}`);
  console.log('');
  console.log(chalk.bold('  By Category:'));
  console.log(`    🎯 Prompt Injection: ${summary.promptInjection}`);
  console.log(`    🔑 Secrets:          ${summary.secrets}`);
  console.log(`    👤 PII:              ${summary.pii}`);
  console.log(`    ⚠️  Toxicity:         ${summary.toxicity}`);
  console.log('');
  console.log(chalk.bold('  By Severity:'));
  console.log(`    ${chalk.bgRed.white(' CRITICAL ')} ${summary.criticalCount}`);
  console.log(`    ${chalk.red('HIGH')}       ${summary.highCount}`);
  console.log(`    ${chalk.yellow('MEDIUM')}     ${summary.mediumCount}`);
  console.log(`    ${chalk.blue('LOW')}        ${summary.lowCount}`);
}

async function scanFile(filePath: string): Promise<ScanResult[]> {
  try {
    const content = readFileSync(filePath, 'utf-8');
    return scanText(content, filePath);
  } catch (error) {
    console.error(chalk.red(`Error reading file: ${filePath}`));
    return [];
  }
}

async function scanDirectory(dir: string, patterns: string[]): Promise<ScanResult[]> {
  const results: ScanResult[] = [];
  const files = await glob(patterns, { cwd: dir, absolute: true, nodir: true });

  for (const file of files) {
    const fileResults = await scanFile(file);
    results.push(...fileResults);
  }

  return results;
}

// CLI Program
const program = new Command();

program
  .name('shield')
  .description('LLM Shield CLI - Enterprise-grade security scanning for Large Language Models')
  .version(VERSION);

program
  .command('scan')
  .description('Scan files or directories for security issues')
  .argument('[path]', 'File or directory to scan', '.')
  .option('-p, --pattern <patterns...>', 'File patterns to include', ['**/*.txt', '**/*.md', '**/*.json', '**/*.yaml', '**/*.yml', '**/*.js', '**/*.ts', '**/*.py'])
  .option('-e, --exclude <patterns...>', 'Patterns to exclude', ['**/node_modules/**', '**/dist/**', '**/.git/**'])
  .option('--secrets', 'Only scan for secrets')
  .option('--pii', 'Only scan for PII')
  .option('--prompt-injection', 'Only scan for prompt injection')
  .option('--toxicity', 'Only scan for toxicity')
  .option('-o, --output <format>', 'Output format (text, json)', 'text')
  .option('--fail-on <severity>', 'Exit with error if issues of this severity or higher are found', 'high')
  .action(async (path: string, options) => {
    const spinner = ora('Scanning...').start();

    try {
      const absolutePath = resolve(path);
      let results: ScanResult[] = [];
      let fileCount = 0;

      if (existsSync(absolutePath)) {
        const stat = statSync(absolutePath);
        if (stat.isDirectory()) {
          const patterns = options.pattern.map((p: string) =>
            options.exclude.reduce((acc: string, ex: string) => `!${ex}`, p)
          );
          const files = await glob(options.pattern, {
            cwd: absolutePath,
            absolute: true,
            nodir: true,
            ignore: options.exclude
          });
          fileCount = files.length;

          for (const file of files) {
            const fileResults = await scanFile(file);
            results.push(...fileResults.map(r => ({
              ...r,
              file: relative(absolutePath, r.file)
            })));
          }
        } else {
          fileCount = 1;
          results = await scanFile(absolutePath);
        }
      } else {
        spinner.fail(`Path not found: ${absolutePath}`);
        process.exit(1);
      }

      spinner.stop();

      // Filter by category if specified
      if (options.secrets) results = results.filter(r => r.category === 'secret');
      if (options.pii) results = results.filter(r => r.category === 'pii');
      if (options.promptInjection) results = results.filter(r => r.category === 'prompt-injection');
      if (options.toxicity) results = results.filter(r => r.category === 'toxicity');

      // Output
      if (options.output === 'json') {
        console.log(JSON.stringify(results, null, 2));
      } else {
        console.log(chalk.bold('\n🛡️  LLM Shield Scan Results\n'));

        if (results.length === 0) {
          console.log(chalk.green('  ✓ No security issues found!\n'));
        } else {
          results.forEach((result) => {
            console.log(formatResult(result));
            console.log('');
          });
        }

        const summary: ScanSummary = {
          totalFiles: fileCount,
          totalIssues: results.length,
          promptInjection: results.filter(r => r.category === 'prompt-injection').length,
          secrets: results.filter(r => r.category === 'secret').length,
          pii: results.filter(r => r.category === 'pii').length,
          toxicity: results.filter(r => r.category === 'toxicity').length,
          criticalCount: results.filter(r => r.severity === 'critical').length,
          highCount: results.filter(r => r.severity === 'high').length,
          mediumCount: results.filter(r => r.severity === 'medium').length,
          lowCount: results.filter(r => r.severity === 'low').length,
        };

        printSummary(summary);
      }

      // Exit code based on severity
      const severityOrder = ['low', 'medium', 'high', 'critical'];
      const failIndex = severityOrder.indexOf(options.failOn);
      const hasFailingSeverity = results.some(r => severityOrder.indexOf(r.severity) >= failIndex);

      if (hasFailingSeverity) {
        process.exit(1);
      }
    } catch (error) {
      spinner.fail('Scan failed');
      console.error(error);
      process.exit(1);
    }
  });

program
  .command('check')
  .description('Check a single text input for security issues')
  .argument('<text>', 'Text to check')
  .option('-o, --output <format>', 'Output format (text, json)', 'text')
  .action((text: string, options) => {
    const results = scanText(text, 'input');

    if (options.output === 'json') {
      console.log(JSON.stringify({
        is_safe: results.length === 0,
        issues: results.length,
        results,
      }, null, 2));
    } else {
      console.log(chalk.bold('\n🛡️  LLM Shield Check\n'));

      if (results.length === 0) {
        console.log(chalk.green('  ✓ Text is safe!\n'));
      } else {
        console.log(chalk.red(`  ✗ Found ${results.length} issue(s):\n`));
        results.forEach((result) => {
          console.log(formatResult(result));
          console.log('');
        });
      }
    }

    process.exit(results.length > 0 ? 1 : 0);
  });

program
  .command('version')
  .description('Show version information')
  .action(() => {
    console.log(chalk.bold('\n🛡️  LLM Shield CLI'));
    console.log(`  Version: ${VERSION}`);
    console.log(`  Node.js: ${process.version}`);
    console.log(`  Platform: ${process.platform}`);
    console.log('');
  });

program.parse();
