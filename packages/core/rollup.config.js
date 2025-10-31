import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import terser from '@rollup/plugin-terser';
import copy from 'rollup-plugin-copy';
import filesize from 'rollup-plugin-filesize';

const production = !process.env.ROLLUP_WATCH;

const plugins = [
  resolve({
    browser: false,
    preferBuiltins: true,
  }),
  commonjs(),
  typescript({
    tsconfig: './tsconfig.json',
    declaration: false, // Types generated separately
  }),
  production && terser({
    compress: {
      passes: 2,
    },
    mangle: {
      properties: false,
    },
  }),
  filesize({
    showGzippedSize: true,
  }),
];

export default [
  // ESM build (main entry)
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.mjs',
      format: 'es',
      sourcemap: true,
    },
    plugins: [
      ...plugins,
      copy({
        targets: [
          { src: '../../README.md', dest: 'dist' },
          { src: '../../LICENSE', dest: 'dist' },
        ],
      }),
    ],
    external: [],
  },

  // CJS build (Node.js compatibility)
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.cjs',
      format: 'cjs',
      sourcemap: true,
      exports: 'named',
    },
    plugins,
    external: [],
  },

  // Node.js optimized build
  {
    input: 'src/node/index.ts',
    output: [
      {
        file: 'dist/node/index.mjs',
        format: 'es',
        sourcemap: true,
      },
      {
        file: 'dist/node/index.cjs',
        format: 'cjs',
        sourcemap: true,
        exports: 'named',
      },
    ],
    plugins,
    external: [],
  },

  // Browser optimized build
  {
    input: 'src/browser/index.ts',
    output: [
      {
        file: 'dist/browser/index.mjs',
        format: 'es',
        sourcemap: true,
      },
      {
        file: 'dist/browser/index.cjs',
        format: 'cjs',
        sourcemap: true,
        exports: 'named',
      },
      {
        file: 'dist/browser/llm-shield.umd.js',
        format: 'umd',
        name: 'LLMShield',
        sourcemap: true,
      },
    ],
    plugins: [
      resolve({
        browser: true,
        preferBuiltins: false,
      }),
      commonjs(),
      typescript({
        tsconfig: './tsconfig.json',
        declaration: false,
      }),
      production && terser(),
      filesize({
        showGzippedSize: true,
      }),
    ],
    external: [],
  },

  // Edge runtime build
  {
    input: 'src/edge/index.ts',
    output: {
      file: 'dist/edge/index.mjs',
      format: 'es',
      sourcemap: true,
    },
    plugins,
    external: [],
  },
];
