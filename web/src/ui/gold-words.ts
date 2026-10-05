/** Presentation only: persisted exit/ledger words remain stable for accounting. */
export const goldWords = (text: string): string => text.replace(/\bbanked\b/gi, (word) => word === word.toUpperCase() ? "COLLECTED" : "collected").replace(/\bbank (deposit|withdraw)\b/g, "savings $1");
