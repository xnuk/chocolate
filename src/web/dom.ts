const getId = (id: string) => document.getElementById(id) as HTMLElement

export const source = getId('source') as HTMLTextAreaElement
export const sourceView = getId('source-view') as HTMLDivElement
export const stack = getId('stack') as HTMLUListElement
export const buttonRow = getId('button-row') as HTMLDivElement
export const buttonNext = getId('button-next') as HTMLButtonElement
export const buttonRun = getId('button-run') as HTMLButtonElement
export const buttonStop = getId('button-stop') as HTMLButtonElement
export const input = getId('input') as HTMLTextAreaElement
export const output = getId('output') as HTMLPreElement
export const runMetadata = getId('run-metadata') as HTMLDivElement
export const main = document.getElementsByTagName('main')[0] as HTMLElement
export const error = getId('error') as HTMLDivElement
