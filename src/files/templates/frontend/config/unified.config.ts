// Unified configuration data store
// This file serves as a single source of truth for all configuration values
// Other tools can import and use these values to generate their specific configs

export interface UnifiedConfig {
  // Project metadata
  project: {
    name: string
    version: string
    description: string
  }

  // Development server configuration
  devServer: {
    port: number
    host: string
    https: boolean
  }

  // Build configuration
  build: {
    target: string
    sourcemap: boolean
    outDir: string
    assetsDir: string
    rollupOptions: {
      output: {
        manualChunks: Record<string, string[]>
      }
    }
  }

  // TypeScript configuration
  typescript: {
    target: string
    module: string
    moduleResolution: string
    strict: boolean
    skipLibCheck: boolean
    noUncheckedIndexedAccess: boolean
    allowImportingTsExtensions: boolean
    isolatedModules: boolean
    verbatimModuleSyntax: boolean
    jsx: string
    lib: string[]
    types: string[]
  }

  // Testing configuration
  testing: {
    unit: {
      environment: string
      globals: boolean
      exclude: string[]
    }
    e2e: {
      timeout: number
      retries: number
      workers: number
      browsers: string[]
    }
  }

  // Code quality configuration
  codeQuality: {
    formatting: {
      semi: boolean
      singleQuote: boolean
      printWidth: number
      tabWidth: number
      useTabs: boolean
      trailingComma: string
      bracketSpacing: boolean
      arrowParens: string
      endOfLine: string
    }
    linting: {
      vue: {
        multiWordComponentNames: boolean
        noUnusedVars: boolean
        noUnusedComponents: boolean
      }
      typescript: {
        noUnusedVars: boolean
        explicitFunctionReturnType: boolean
        explicitModuleBoundaryTypes: boolean
        noExplicitAny: string
      }
      general: {
        noConsole: string
        noDebugger: string
        preferConst: boolean
        noVar: boolean
        objectShorthand: boolean
        preferTemplate: boolean
      }
    }
  }

  // Path aliases
  aliases: Record<string, string>

  // Auto-import configuration
  autoImport: {
    imports: string[]
    resolvers: string[]
    dirs: string[]
    vueTemplate: boolean
    dts: boolean
    eslintrc: boolean
  }

  // OpenAPI/HeyAPI configuration
  openapi: {
    input: string
    output: {
      path: string
      // Add other output options as needed
    },
    plugins: ReadonlyArray<
      | string
      | {
          [K in string]: any & {
            name: K;
          };
        }[string]
    >
    watch: boolean
  }
}

// Default configuration values
export const defaultConfig: UnifiedConfig = {
  project: {
    name: 'frontend',
    version: '0.0.0',
    description: 'Vue.js frontend for Rext demo project'
  },

  devServer: {
    port: 5173,
    host: 'localhost',
    https: false
  },

  build: {
    target: 'ESNext',
    sourcemap: true,
    outDir: 'dist',
    assetsDir: 'assets',
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['vue', 'vue-router', 'pinia'],
          ui: ['element-plus']
        }
      }
    }
  },

  typescript: {
    target: 'ESNext',
    module: 'ESNext',
    moduleResolution: 'Node',
    strict: true,
    skipLibCheck: true,
    noUncheckedIndexedAccess: true,
    allowImportingTsExtensions: true,
    isolatedModules: true,
    verbatimModuleSyntax: true,
    jsx: 'preserve',
    lib: ['ESNext', 'DOM', 'DOM.Iterable'],
    types: ['node', 'jsdom']
  },

  testing: {
    unit: {
      environment: 'jsdom',
      globals: true,
      exclude: ['e2e/**', 'node_modules/**', 'dist/**']
    },
    e2e: {
      timeout: 30000,
      retries: 2,
      workers: 1,
      browsers: ['chromium', 'firefox', 'webkit']
    }
  },

  codeQuality: {
    formatting: {
      semi: false,
      singleQuote: true,
      printWidth: 100,
      tabWidth: 2,
      useTabs: false,
      trailingComma: 'es5',
      bracketSpacing: true,
      arrowParens: 'avoid',
      endOfLine: 'lf'
    },
    linting: {
      vue: {
        multiWordComponentNames: false,
        noUnusedVars: true,
        noUnusedComponents: true
      },
      typescript: {
        noUnusedVars: true,
        explicitFunctionReturnType: false,
        explicitModuleBoundaryTypes: false,
        noExplicitAny: 'warn'
      },
      general: {
        noConsole: 'warn',
        noDebugger: 'warn',
        preferConst: true,
        noVar: true,
        objectShorthand: true,
        preferTemplate: true
      }
    }
  },

  aliases: {
    '@/appearance': './src/appearance',
    '@/bridge': './src/bridge',
    '@/components': './src/appearance/components',
    '@/views': './src/appearance/views',
    '@/pages': './src/appearance/pages',
    '@/layouts': './src/appearance/layouts',
    '@/styles': './src/appearance/styles',
    '@/api': './src/bridge/api',
    '@/stores': './src/bridge/stores',
    '@/types': './src/bridge/types',
    '@/router': './src/bridge/router',
    '@': './src'
  },

  autoImport: {
    imports: ['vue', 'vue-router', 'pinia'],
    resolvers: ['ElementPlusResolver'],
    dirs: [
      './src/composables/**',
      './src/directives/**',
      './src/bridge/client/index.ts'
    ],
    vueTemplate: true,
    dts: true,
    eslintrc: true
  },

  openapi: {
    input: 'http://localhost:3000/api-docs/openapi.json',
    output: {
      path: 'src/bridge/client'
    },
    plugins: [
      'zod',
      {
        name: '@hey-api/sdk', 
        requests: true,
        responses: true,
        definitions: true,
        metadata: true,
        validator: true, 
      },
    ],
    watch: true
  }
}

// Environment-specific configuration overrides
export const getConfig = (env?: { NODE_ENV?: string; CI?: string }): UnifiedConfig => {
  const config = { ...defaultConfig }
  const environment = env || {}

  // Override based on environment
  if (environment.NODE_ENV === 'production') {
    config.codeQuality.linting.general.noConsole = 'error'
    config.codeQuality.linting.general.noDebugger = 'error'
    config.build.sourcemap = false
  }

  // Override based on CI environment
  if (environment.CI) {
    config.testing.e2e.retries = 2
    config.testing.e2e.workers = 1
  }

  return config
}

// Utility functions for generating specific configs
export const generateViteConfig = () => {
  const config = getConfig()
  return {
    server: {
      port: config.devServer.port,
      host: config.devServer.host,
      https: config.devServer.https
    },
    build: config.build,
    test: {
      environment: config.testing.unit.environment,
      globals: config.testing.unit.globals,
      exclude: config.testing.unit.exclude
    }
  }
}

export const generateTypeScriptConfig = () => {
  const config = getConfig()
  return {
    compilerOptions: config.typescript,
    paths: config.aliases
  }
}

export const generateESLintConfig = () => {
  const config = getConfig()
  return {
    rules: {
      // Formatting rules
      'semi': ['error', config.codeQuality.formatting.semi ? 'always' : 'never'],
      'quotes': ['error', config.codeQuality.formatting.singleQuote ? 'single' : 'double'],
      'printWidth': ['error', config.codeQuality.formatting.printWidth],
      'tabWidth': ['error', config.codeQuality.formatting.tabWidth],
      'useTabs': ['error', config.codeQuality.formatting.useTabs],
      'trailingComma': ['error', config.codeQuality.formatting.trailingComma],
      'bracketSpacing': ['error', config.codeQuality.formatting.bracketSpacing],
      'arrowParens': ['error', config.codeQuality.formatting.arrowParens],
      'endOfLine': ['error', config.codeQuality.formatting.endOfLine],

      // Vue rules
      'vue/multi-word-component-names': config.codeQuality.linting.vue.multiWordComponentNames ? 'error' : 'off',
      'vue/no-unused-vars': config.codeQuality.linting.vue.noUnusedVars ? 'error' : 'off',
      'vue/no-unused-components': config.codeQuality.linting.vue.noUnusedComponents ? 'error' : 'off',

      // TypeScript rules
      '@typescript-eslint/no-unused-vars': config.codeQuality.linting.typescript.noUnusedVars ? ['error', { 'argsIgnorePattern': '^_' }] : 'off',
      '@typescript-eslint/explicit-function-return-type': config.codeQuality.linting.typescript.explicitFunctionReturnType ? 'error' : 'off',
      '@typescript-eslint/explicit-module-boundary-types': config.codeQuality.linting.typescript.explicitModuleBoundaryTypes ? 'error' : 'off',
      '@typescript-eslint/no-explicit-any': config.codeQuality.linting.typescript.noExplicitAny,

      // General rules
      'no-console': config.codeQuality.linting.general.noConsole,
      'no-debugger': config.codeQuality.linting.general.noDebugger,
      'prefer-const': config.codeQuality.linting.general.preferConst ? 'error' : 'off',
      'no-var': config.codeQuality.linting.general.noVar ? 'error' : 'off',
      'object-shorthand': config.codeQuality.linting.general.objectShorthand ? 'error' : 'off',
      'prefer-template': config.codeQuality.linting.general.preferTemplate ? 'error' : 'off'
    }
  }
}