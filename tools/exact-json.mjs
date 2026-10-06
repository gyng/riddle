// A lossless comparison view for saved Rust integers. Never load this view into
// the engine: retain and use the original JSON string for saves and checkpoints.
export function parseExactJSON(text) {
  return JSON.parse(text, (_key, value, context) => {
    if (typeof value !== 'number' || Number.isSafeInteger(value)) return value;
    if (!context?.source) throw new Error('Exact JSON comparison requires JSON.parse source context');
    return /^-?\d+$/.test(context.source) ? { $integer: context.source } : value;
  });
}
