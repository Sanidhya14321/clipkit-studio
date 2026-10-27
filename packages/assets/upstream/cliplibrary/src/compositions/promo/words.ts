export const wordsOf = (line: string): string[] =>
  line.split(/\s+/).filter((word) => word.length > 0);

export const normalize = (word: string): string => word.replace(/[.,!?;:]/g, '').toLowerCase();

export const highlightSet = (highlight: string): ReadonlySet<string> =>
  new Set(wordsOf(highlight).map(normalize));

export const linesOf = (text: string): string[][] =>
  text
    .split('\n')
    .map(wordsOf)
    .filter((line) => line.length > 0);
