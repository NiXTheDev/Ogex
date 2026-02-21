/**
 * Test file for Ogex WASM bindings - Node.js version
 * 
 * Run with:
 *   node test_ogex_node.mjs
 */

import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

async function main() {
  // Load WASM module manually
  const wasmPath = join(__dirname, 'ogex', 'pkg', 'ogex_bg.wasm');
  const wasmBuffer = await readFile(wasmPath);
  const wasmModule = await WebAssembly.compile(wasmBuffer);
  
  // Load the JS bindings
  const { default: init, JsRegex } = await import('./ogex/pkg/ogex_bg.js');
  
  // Initialize WASM
  const wasmInstance = await WebAssembly.instantiate(wasmModule, {});
  
  console.log("=== Ogex WASM Tests ===\n");
  console.log("Note: WASM initialization may require additional setup.");
  console.log("For full testing, use a bundler like webpack or vite.\n");
  
  // Basic test without WASM initialization
  console.log("Test: Module structure");
  console.log(`  JsRegex class exists: ${typeof JsRegex === 'function'}`);
  
  console.log("\nℹ️  For full WASM testing, run:");
  console.log("   1. npm install in ogex/pkg/");
  console.log("   2. Use a bundler (vite/webpack) or test in browser");
}

main().catch(console.error);
