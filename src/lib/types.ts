import { SvelteComponent } from 'svelte'

export interface BlockInfo {
    name: string;
    actions: ActionInfo[];
    done: boolean;
}

export interface ActionInfo {
    order: number;
    type: SvelteComponent;
    blocking: boolean;
    opt?: object;
}
