import js from '@eslint/js';
import ts from 'typescript-eslint';
export default ts.config(
  {
    ignores: [
      '**/node_modules/**',
      '**/dist/**',
      '**/out/**',
      '**/target/**',
      '**/.venv/**',
      '**/bin/**',
      '**/obj/**',
      '.runner/**',
      'reports/**',
    ],
  },
  {
    files: ['**/*.ts', '**/*.tsx'],
    extends: [js.configs.recommended, ...ts.configs.recommended],
    rules: { '@typescript-eslint/no-explicit-any': 'error' },
  },
);
