#!/usr/bin/env node
// Fast actual saved-town comparisons; diagnostic only, never a balance gate.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';
import { readDebug } from './debug-export.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..'), sha = b => createHash('sha256').update(b).digest('hex');
function run(cmd, args, env) {
    const r = spawnSync(cmd, args, { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, env });
    if (r.error || r.status !== 0)
        throw Error(r.error?.message || r.signal || r.stderr || r.stdout || `${cmd} failed`);
    return r.stdout;
}
async function main() {
    const args = process.argv.slice(2);
    if (args.includes('--help')) {
        console.log('usage: node tools/choice-check.mjs SAVE_OR_DUMP --choices ID[:SLOT],... [--hours 8] [--out NEW_DIR] [--balance PROFILE] [--upgrades health,armour,damage,all]\nBaseline and each choice replay the same saved Session. Diagnostic only; not a balance gate.');
        return;
    }
    const save = args.shift();
    if (!save || save.startsWith('-'))
        throw Error('supply a saved town');
    let choices, hours = 8, out, balance, upgrades;
    for (let i = 0; i < args.length; i += 2) {
        const k = args[i], v = args[i + 1];
        if (!v || v.startsWith('--'))
            throw Error(`missing value for ${k}`);
        if (k === '--choices')
            choices = v;
        else if (k === '--hours')
            hours = Number(v);
        else if (k === '--out')
            out = resolve(v);
        else if (k === '--balance')
            balance = v;
        else if (k === '--upgrades')
            upgrades = v.split(',');
        else
            throw Error(`unknown option ${k}`);
    }
    if (!Number.isSafeInteger(hours) || hours < 1 || hours > 336)
        throw Error('hours must be an integer from 1 to 336');
    if (!choices)
        throw Error('supply --choices');
    const parsed = choices.split(',').map(v => { if (!/^[a-z_]+(?::\d+)?$/.test(v))
        throw Error(`invalid choice ${v}`); const [id, slot = '0'] = v.split(':'); if (!Number.isSafeInteger(Number(slot)))
        throw Error('invalid slot'); return `${id}:${Number(slot)}`; });
    if (new Set(parsed).size !== parsed.length)
        throw Error('duplicate choice');
    if (upgrades && (upgrades.some(id => !['health', 'armour', 'damage', 'all'].includes(id)) || new Set(upgrades).size !== upgrades.length))
        throw Error('unknown or duplicate upgrade path');
    if (out && existsSync(out))
        throw Error('output directory already exists; refusing to replace it');
    const source = readFileSync(save), inputHash = sha(source), env = { ...process.env };
    const header = JSON.parse(source.toString('utf8'));
    let input = source, provenance = null;
    if (header && typeof header === 'object' && 'format' in header) {
        const { bundle } = readDebug(source.toString('utf8'));
        input = Buffer.from(bundle.save.engine, 'utf8');
        provenance = { format: bundle.format, schema: bundle.schema, build: bundle.build,
            capture: bundle.capture, captured_at: bundle.captured_at };
    }
    const engineInputHash = sha(input);
    if (balance)
        env.RIDDLE_BALANCE_JSON = readFileSync(balance, 'utf8');
    else
        delete env.RIDDLE_BALANCE_JSON;
    const started = performance.now(), built = run('cargo', ['build', '-q', '--profile', 'fast', '-p', 'riddle-core', '--example', 'choice_check', ...(balance ? ['--features', 'dev-balance'] : []), '--message-format=json'], env);
    const bin = built.split('\n').filter(Boolean).map(s => JSON.parse(s)).find(m => m.reason === 'compiler-artifact' && m.target.name === 'choice_check' && m.executable)?.executable;
    if (!bin)
        throw Error('Cargo did not report the diagnostic executable');
    const binary = readFileSync(bin), binaryHash = sha(binary);
    const buildSeconds = (performance.now() - started) / 1000, temp = mkdtempSync(join(tmpdir(), 'riddle-choice-'));
    try {
        const snapshot = join(temp, 'input.json'), resultPath = join(temp, 'result.json'), executable = join(temp, 'choice_check');
        writeFileSync(snapshot, input);
        // Pin the actual executable as well as the save: another Cargo build
        // may replace the shared target path while comparisons are running.
        writeFileSync(executable, binary);
        chmodSync(executable, 0o755);
        run(executable, [snapshot, String(hours), parsed.join(','), resultPath, ...(upgrades ? [upgrades.join(',')] : [])], env);
        const result = JSON.parse(readFileSync(resultPath));
        if (sha(readFileSync(save)) !== inputHash)
            throw Error('input changed during diagnostic');
        if (result.version !== (upgrades ? 2 : 1) || result.cases.length !== (parsed.length + 1) * (1 + (upgrades?.length ?? 0)))
            throw Error('incomplete diagnostic');
        for (const c of result.cases) {
            const r = c.report, deaths = r.deaths.reduce((n, d) => n + d.n, 0);
            console.log(`${c.upgrade ? `${c.upgrade.path} (${c.upgrade.spent} Legacy) / ` : ''}${c.choice ? `${c.choice.id}:${c.choice.slot}` : 'baseline'}: ${r.runs} runs · D${r.deepest ?? '—'} · $${r.gold?.home ?? 0} home · ${deaths} ${deaths === 1 ? 'death' : 'deaths'} · ${c.seconds.toFixed(3)}s`);
        }
        if (out) {
            mkdirSync(dirname(out), { recursive: true });
            mkdirSync(out, { recursive: false });
            const files = [];
            for (const [i, c] of result.cases.entries()) {
                const file = `${i}-${c.upgrade ? `${c.upgrade.path}-` : ''}${c.choice?.id ?? 'baseline'}.json`, body = JSON.stringify(c);
                writeFileSync(join(out, file), body);
                files.push({ file, sha256: sha(body) });
            }
            writeFileSync(join(out, 'manifest.json'), JSON.stringify({ version: result.version, ...(upgrades ? { upgrades } : {}), input: resolve(save), inputSha256: inputHash, engineInputSha256: engineInputHash, sourceCapture: provenance, binarySha256: binaryHash, balanceSha256: balance ? sha(env.RIDDLE_BALANCE_JSON) : null, selected: result.selected, slots: result.slots, hours, choices: parsed, cases: files, buildSeconds, totalSeconds: (performance.now() - started) / 1000 }, null, 2));
        }
        console.log(`choice-check: ${result.slots} active ${result.slots === 1 ? 'hero' : 'heroes'} · build ${buildSeconds.toFixed(2)}s · total ${((performance.now() - started) / 1000).toFixed(2)}s · diagnostic only`);
    }
    finally {
        rmSync(temp, { recursive: true, force: true });
    }
}
main().catch(e => { console.error(`choice-check: ${e.message}`); process.exitCode = 1; });
