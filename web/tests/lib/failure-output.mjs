/** Keep actionable failures and Playwright targets without printing the whole walk. */
export function failureOutput(output, limit = 20) {
  const lines = output.replace(/\x1b\[[0-9;]*m/g, '').split('\n');
  const context = line => !/^ok(?:\s|$)/.test(line) && (/error|aborted/i.test(line)
    || /^\s*- waiting for (locator|(?:getBy\w+)\()/.test(line)
    || /intercepts pointer events|element is not (visible|enabled)|strict mode violation/.test(line));
  // A long assertion list must not crowd the blocking locator out of the summary.
  return [...lines.filter(context), ...lines.filter(line => !context(line)
    && /^(FAIL|fail|not ok|Error|\s+at )/.test(line))].slice(0, limit);
}
