/** Presentation only: persisted exit/ledger words remain stable for accounting. */
export const goldWords = (text: string): string => text.replace(/\bbanked\b/gi, (word) => word === word.toUpperCase() ? "COLLECTED" : "collected").replace(/\bbank (deposit|withdraw)\b/g, "savings $1");
/** Cut 122 §3 (blind 9621b19: `slain · King` read as the King slain): a death's reason names who killed him — `died to the King`,
 *  `died to jackal` — never `slain · X`, which reads as X slain. */
export const deathWhy = (why: string): string => {
  const m = /^slain · (.+)$/.exec(why.trim()); if (!m) return why;
  const who = m[1].replace(/_/g, " ").trim();
  return /\bking\b/i.test(who) ? /* copy:death_line */ "died to the King" : /^[A-Z]/.test(who) ? /* copy:death_line */ `died to the ${who}` : /* copy:death_line */ `died to ${who}`;
};
