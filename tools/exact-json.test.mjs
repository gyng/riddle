import test from 'node:test';import assert from 'node:assert/strict';
import {parseExactJSON} from './exact-json.mjs';
test('save comparison detects adjacent integers that Number rounds together',()=>{
 const a='{"rng":9007199254740992}',b='{"rng":9007199254740993}';
 assert.deepEqual(JSON.parse(a),JSON.parse(b));assert.notDeepEqual(parseExactJSON(a),parseExactJSON(b));
 assert.deepEqual(parseExactJSON('{"max":18446744073709551615,"negative":-9007199254740993}'),{max:{$integer:'18446744073709551615'},negative:{$integer:'-9007199254740993'}});
});
test('comparison preserves safe counters, floats, arrays and quoted seed-like text',()=>{
 const text='{"safe":9007199254740991,"text":"18446744073709551615 \\"quoted\\"","ratio":0.375,"largeFloat":1e20,"values":[null,true,-4]}';
 assert.deepEqual(parseExactJSON(text),JSON.parse(text));
 assert.deepEqual(parseExactJSON('{"f":9007199254740992.0}'),{f:9007199254740992});
});
