/**
 * Test file for Ogex WASM bindings
 * 
 * Run with:
 *   node --experimental-strip-types test_ogex.ts
 *   deno run --allow-read --allow-net test_ogex.ts  
 *   bun test_ogex.ts
 */

// Import the WASM module
import init, { JsRegex } from "./ogex/pkg/ogex.js";

async function main() {
  // Initialize WASM
  await init();

  console.log("=== Ogex WASM Tests ===\n");

  // Test 1: Basic match
  console.log("Test 1: Basic match");
  const regex1 = new JsRegex("hello");
  console.log(`  "hello world" matches: ${regex1.is_match("hello world")}`);
  console.log(`  "goodbye" matches: ${regex1.is_match("goodbye")}`);

  // Test 2: Named groups
  console.log("\nTest 2: Named groups");
  const regex2 = new JsRegex("(name:\\w+)");
  const match2 = regex2.find("hello name:John Smith");
  if (match2) {
    console.log(`  Match: "${match2.text}"`);
    console.log(`  Named group 'name': "${match2.named_group("name")}"`);
  }

  // Test 3: Relative backreference \g{-1}
  console.log("\nTest 3: Relative backreference \\g{-1}");
  const regex3 = new JsRegex("(a)(b)\\g{-1}");
  console.log(`  "abb" matches: ${regex3.is_match("abb")}`);
  console.log(`  "aba" matches: ${regex3.is_match("aba")}`);

  // Test 4: \G literal in pattern
  console.log("\nTest 4: \\G literal in pattern");
  const regex4 = new JsRegex("\\G");
  console.log(`  "G" matches: ${regex4.is_match("G")}`);
  console.log(`  "g" matches: ${regex4.is_match("g")}`);

  // Test 5: Find all
  console.log("\nTest 5: Find all");
  const regex5 = new JsRegex("\\d+");
  const matches5 = regex5.find_all("abc 123 def 456 ghi 789");
  console.log(`  Found ${matches5.length} matches:`);
  for (let i = 0; i < matches5.length; i++) {
    const m = matches5[i];
    console.log(`    ${i + 1}: "${m.text}" at ${m.start}-${m.end}`);
  }

  // Test 6: Transpile
  console.log("\nTest 6: Transpile");
  const transpiled = JsRegex.transpile("(name:\\w+)(email:\\w+)");
  console.log(`  "(name:\\w+)(email:\\w+)" -> "${transpiled}"`);

  console.log("\n✅ All tests completed!");
}

main().catch(console.error);
