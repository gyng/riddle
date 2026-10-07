/** Player-facing names; stable engine IDs remain compatible with existing saves. */
export function gunName(kind: string): string {
  return kind === 'long_gun' ? /* copy:label */ 'Rifle' : kind === 'short_gun' ? /* copy:label */ 'Scattergun' : kind.replace(/_/g, ' ');
}
