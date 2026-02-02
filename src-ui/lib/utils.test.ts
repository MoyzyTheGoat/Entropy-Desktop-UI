import { describe, it, expect, vi } from 'vitest';
import { parseLinkPreview } from './utils';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn()
}));

// Mock svelte/store
vi.mock('svelte/store', () => ({
    get: vi.fn(() => ({
        privacySettings: {
            routingMode: 'direct',
            proxyUrl: ''
        }
    })),
    writable: vi.fn((val) => ({
        subscribe: vi.fn((cb) => {
            cb(val);
            return () => { };
        }),
        set: vi.fn(),
        update: vi.fn()
    }))
}));

import { invoke } from '@tauri-apps/api/core';

describe('utils.ts', () => {
    describe('parseLinkPreview', () => {
        it('should return null if no URL is found', async () => {
            const result = await parseLinkPreview('Hello world');
            expect(result).toBeNull();
        });

        it('should call invoke with the found URL', async () => {
            const mockPreview = { title: 'Test', siteName: 'test.com' };
            (invoke as any).mockResolvedValue(mockPreview);

            const result = await parseLinkPreview('Check this out: https://example.com');
            expect(invoke).toHaveBeenCalledWith('get_link_preview', expect.objectContaining({
                url: 'https://example.com'
            }));
            expect(result).toEqual(mockPreview);
        });

        it('should handle errors gracefully', async () => {
            (invoke as any).mockRejectedValue(new Error('Fail'));
            const result = await parseLinkPreview('https://error.com');
            expect(result).toEqual({
                url: 'https://error.com',
                title: 'https://error.com',
                siteName: 'error.com'
            });
        });
    });
});
