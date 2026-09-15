#!/usr/bin/env node
// Copy lint: player-facing string literals in web/src/ui/** against eval/copy-budgets.json.
//
// Convention: a `/* copy:<surface> */` comment tags the NEXT string literal, or — when the next
// bracket comes first — every string literal inside that `[...]` / `{...}` (for label tables).
// Template literals count only their static text (`${...}` is data, not copy).
//   maxWords     — a word is a whitespace-separated token containing a letter ("D7", "31%", "→" do not count as words)
//   noSentences  — no terminal `.` `!` `?` after a word, and never "you", "your", "click", "tap", "this shows"
// The forbidden words are checked in EVERY literal in ui/** (tagged or not). An untagged literal that
// looks like prose (two or more words and a capital letter, or a terminal stop) is a violation: tag it,
// or keep chrome to nouns/glyphs. `/* copy:none */` marks a literal as not player-facing.
// Exit code 1 on any violation.
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const BUDGETS = JSON.parse(readFileSync(join(ROOT, "eval/copy-budgets.json"), "utf8")).surfaces;
const UI_DIR = join(ROOT, "web/src/ui");
const FORBIDDEN = [/\byou\b/i, /\byour\b/i, /\bclick\b/i, /\btap\b/i, /\bthis shows\b/i];

function walk(dir) {
  const out = [];
  for (const f of readdirSync(dir)) { const p = join(dir, f); if (statSync(p).isDirectory()) out.push(...walk(p)); else if (/\.(ts|js|mts|mjs)$/.test(f)) out.push(p); }
  return out;
}

// Tokenizer: yields {type:'comment'|'string'|'punct', text, line}. Strings carry .literal (static text only).
function tokenize(src) {
  const toks = []; let i = 0, line = 1; const n = src.length;
  const REGEX_PREV = /[(,=:[!&|?{};+\-*%<>~^]$|\breturn$|\btypeof$|\bcase$/;
  const push = (type, text, extra = {}) => toks.push({ type, text, line, ...extra });
  let lastSig = "";
  const templateStack = []; // depth counters for ${} nesting
  while (i < n) {
    const c = src[i], d = src[i + 1];
    if (c === "\n") { line++; i++; continue; }
    if (c === "/" && d === "/") { const e = src.indexOf("\n", i); const t = src.slice(i, e < 0 ? n : e); push("comment", t); i += t.length; continue; }
    if (c === "/" && d === "*") { const e = src.indexOf("*/", i + 2); const t = src.slice(i, e < 0 ? n : e + 2); push("comment", t); line += (t.match(/\n/g) || []).length; i += t.length; continue; }
    if (c === '"' || c === "'") {
      let j = i + 1, s = "";
      while (j < n && src[j] !== c) { if (src[j] === "\\") { s += src[j + 1]; j += 2; continue; } s += src[j++]; }
      push("string", src.slice(i, j + 1), { literal: s }); lastSig = "s"; i = j + 1; continue;
    }
    if (c === "`") {
      // template literal, with nested ${ } handled by recursive scan of the expression as plain code
      let j = i + 1, s = "", startLine = line;
      while (j < n && src[j] !== "`") {
        if (src[j] === "\\") { s += src[j + 1]; j += 2; continue; }
        if (src[j] === "$" && src[j + 1] === "{") {
          let depth = 1; j += 2; s += " ";
          while (j < n && depth) { if (src[j] === "{") depth++; else if (src[j] === "}") depth--; else if (src[j] === "`") { /* nested template: skip naively */ let k = src.indexOf("`", j + 1); j = k < 0 ? n : k; } else if (src[j] === "\n") line++; j++; }
          continue;
        }
        if (src[j] === "\n") line++;
        s += src[j++];
      }
      toks.push({ type: "string", text: src.slice(i, j + 1), line: startLine, literal: s }); lastSig = "s"; i = j + 1; continue;
    }
    if (c === "/" && REGEX_PREV.test(lastSig.trimEnd())) {
      let j = i + 1, cls = false;
      while (j < n && (cls || src[j] !== "/")) { if (src[j] === "\\") j++; else if (src[j] === "[") cls = true; else if (src[j] === "]") cls = false; else if (src[j] === "\n") break; j++; }
      j++; while (j < n && /[a-z]/.test(src[j])) j++;
      i = j; lastSig = ")"; continue;
    }
    if ("[]{}()".includes(c)) { push("punct", c); lastSig = c; i++; continue; }
    if (/\s/.test(c)) { i++; continue; }
    // identifier / number / operator run
    let j = i; if (/[A-Za-z0-9_$]/.test(c)) { while (j < n && /[A-Za-z0-9_$]/.test(src[j])) j++; } else { j = i + 1; }
    lastSig = src.slice(i, j); i = j;
  }
  void templateStack;
  return toks;
}

const words = (s) => s.split(/\s+/).filter((w) => /\p{L}/u.test(w));
const hasStop = (s) => /\p{L}[.!?]+$/u.test(s.trim());

function lintFile(file) {
  const src = readFileSync(file, "utf8");
  const toks = tokenize(src);
  const errs = [];
  const rel = relative(ROOT, file);
  const tagged = new Set();
  // assign tags
  for (let i = 0; i < toks.length; i++) {
    const t = toks[i];
    if (t.type !== "comment") continue;
    const m = /copy:([a-z_]+)/.exec(t.text);
    if (!m) continue;
    const surface = m[1];
    if (surface !== "none" && !BUDGETS[surface]) { errs.push(`${rel}:${t.line} unknown surface "${surface}"`); continue; }
    let j = i + 1;
    while (j < toks.length && toks[j].type === "comment") j++;
    if (j >= toks.length) { errs.push(`${rel}:${t.line} dangling copy tag`); continue; }
    if (toks[j].type === "string") { toks[j].surface = surface; tagged.add(toks[j]); continue; }
    // skip to the first bracket or string, whichever first
    while (j < toks.length && !(toks[j].type === "string" || (toks[j].type === "punct" && "[{".includes(toks[j].text)))) j++;
    if (j >= toks.length) { errs.push(`${rel}:${t.line} dangling copy tag`); continue; }
    if (toks[j].type === "string") { toks[j].surface = surface; tagged.add(toks[j]); continue; }
    const open = toks[j].text, close = open === "[" ? "]" : "}"; let depth = 0;
    for (let k = j; k < toks.length; k++) {
      const u = toks[k];
      if (u.type === "punct") { if (u.text === open) depth++; else if (u.text === close) { depth--; if (!depth) break; } }
      else if (u.type === "string") { u.surface = surface; tagged.add(u); }
    }
  }
  for (const t of toks) {
    if (t.type !== "string") continue;
    const lit = t.literal;
    for (const f of FORBIDDEN) if (f.test(lit)) errs.push(`${rel}:${t.line} forbidden word in ${t.text}`);
    if (t.surface === "none") continue;
    if (t.surface) {
      const b = BUDGETS[t.surface]; const w = words(lit);
      if (w.length > b.maxWords) errs.push(`${rel}:${t.line} ${t.surface}: ${w.length} words > ${b.maxWords} in ${t.text}`);
      if (b.noSentences && hasStop(lit)) errs.push(`${rel}:${t.line} ${t.surface}: terminal punctuation in ${t.text}`);
    } else {
      const w = words(lit);
      if ((w.length >= 2 && /\p{Lu}/u.test(lit)) || hasStop(lit)) errs.push(`${rel}:${t.line} untagged copy ${t.text}`);
    }
  }
  return { errs, tagged: tagged.size };
}

let total = 0, taggedN = 0, files = 0;
for (const f of walk(UI_DIR)) {
  const { errs, tagged } = lintFile(f); files++; taggedN += tagged;
  for (const e of errs) { console.error(e); total++; }
}
console.log(`copy-lint: ${files} files, ${taggedN} tagged literals, ${total} violation${total === 1 ? "" : "s"}`);
process.exit(total ? 1 : 0);
