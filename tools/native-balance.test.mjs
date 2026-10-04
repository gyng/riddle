import {NativeHost} from './native-host.mjs';
import test from 'node:test';
import {readFileSync,writeFileSync,mkdtempSync,rmSync} from 'node:fs';
import assert from 'node:assert/strict';
test('live Rust profile reload preserves state and rejects invalid edits', {skip: !process.env.RIDDLE_NATIVE_TEST_BIN}, async () => {
const dir=mkdtempSync('/tmp/riddle-balance-'),file=dir+'/profile.json';
const base=JSON.parse(readFileSync(new URL('../tuning/balance.json', import.meta.url),'utf8'));
writeFileSync(file,JSON.stringify(base));
const h=new NativeHost({binary:process.env.RIDDLE_NATIVE_TEST_BIN,balanceFile:file,threads:1});
try{
 await h.call('a','__bridge',[]);await h.call('a','newLineage',[15]);
 const before=JSON.parse(await h.call('a','save',[]));
 const tuned={...base,steady_heal:[31,36]};writeFileSync(file,JSON.stringify(tuned));
 assert.deepEqual(JSON.parse(await h.call('a','__balance',[])),tuned);
 const after=JSON.parse(await h.call('a','save',[]));
 assert.equal(after.lineage.seed,before.lineage.seed);assert.equal(after.lineage.gold,before.lineage.gold);
 assert.equal(after.lineage.sets[0].rows[0].conds[0].n,31);
 writeFileSync(file,JSON.stringify({...tuned,scar_pct:0}));
 await assert.rejects(h.call('a','save',[]),/invalid balance/);
 writeFileSync(file,JSON.stringify(tuned));assert.equal(JSON.parse(await h.call('a','save',[])).lineage.seed,15);

}finally{h.close();rmSync(dir,{recursive:true,force:true});}

});
