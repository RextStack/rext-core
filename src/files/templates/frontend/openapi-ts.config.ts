import { defineConfig } from '@hey-api/openapi-ts'
import { getConfig } from './config/unified.config'

// Get unified configuration
const config = getConfig()

export default defineConfig({
  input: config.openapi.input,
  output: config.openapi.output,
  plugins: config.openapi.plugins,
  watch: config.openapi.watch,
}) 