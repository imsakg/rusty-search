import { writable } from 'svelte/store';
import type { SourceData } from '$lib/types';

export const sources = writable<SourceData[]>([]);
