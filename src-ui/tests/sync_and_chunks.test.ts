import { describe, it, expect, vi, beforeEach } from 'vitest';
import { handleIncomingMessage } from '../lib/actions/messaging';
import { userStore } from '../lib/stores/user';
import { signalManager } from '../lib/signal_manager';
import { attachmentStore } from '../lib/attachment_store';
import { get } from 'svelte/store';

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(),
}));

vi.mock('../lib/signal_manager', () => ({
    signalManager: {
        decrypt: vi.fn(),
        decryptMediaChunk: vi.fn()
    }
}));

vi.mock('../lib/attachment_store', () => ({
    attachmentStore: {
        put: vi.fn(),
        init: vi.fn()
    }
}));

vi.mock('../lib/actions/message_utils', () => ({
    addMessage: vi.fn((chatId, msg) => {
        userStore.update(s => {
            if (!s.chats[chatId]) s.chats[chatId] = { messages: [], peerHash: chatId, unreadCount: 0, peerAlias: '', isVerified: false, isGroup: false };
            s.chats[chatId].messages.push(msg);
            return s;
        });
    }),
    sendReceipt: vi.fn()
}));

vi.mock('../lib/utils', async () => {
    return {
        fromBase64: (s: string) => new TextEncoder().encode(atob(s)),
        toBase64: (v: Uint8Array) => btoa(String.fromCharCode(...v)),
        fromHex: (h: string) => new Uint8Array(h.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))),
        toHex: (v: Uint8Array) => Array.from(v).map(b => b.toString(16).padStart(2, '0')).join('')
    };
});

// Mock user store with a real writable for testing updates
vi.mock('../lib/stores/user', async () => {
    const { writable: realWritable } = await vi.importActual('svelte/store') as any;
    const store = realWritable({
        identityHash: 'me',
        chats: {},
        blockedHashes: [],
        isConnected: true
    });
    return { userStore: store };
});

describe('Sync and Chunked Media', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        userStore.set({
            identityHash: 'me',
            chats: {
                'friend': { peerHash: 'friend', messages: [], unreadCount: 0, peerAlias: 'Friend' }
            },
            blockedHashes: [],
            isConnected: true
        } as any);
    });

    it('should handle sync_msg by adding it to the correct chat', async () => {
        const syncPayload = {
            type: 'sync_msg',
            destination: 'friend',
            content: 'I sent this from my phone',
            id: 'sync123'
        };

        vi.mocked(signalManager.decrypt).mockResolvedValue({ m: JSON.stringify(syncPayload) });

        // Incoming message from "me" (my other device)
        await handleIncomingMessage({ sender: 'me', message: { type: 1, body: 'encrypted' } } as any);

        const state = get(userStore);
        const chat = state.chats['friend'];
        expect(chat.messages.length).toBe(1);
        expect(chat.messages[0].content).toBe('I sent this from my phone');
        expect(chat.messages[0].isMine).toBe(true);
    });

    it('should handle block_sync by updating blockedHashes', async () => {
        const blockPayload = {
            type: 'block_sync',
            peerHash: 'spammer',
            isBlocked: true
        };

        vi.mocked(signalManager.decrypt).mockResolvedValue({ m: JSON.stringify(blockPayload) });

        await handleIncomingMessage({ sender: 'me', message: { type: 1, body: 'encrypted' } } as any);

        const state = get(userStore);
        expect(state.blockedHashes).toContain('spammer');
    });

    it('should reassemble chunked files correctly', async () => {
        const fileId = 'file123';
        const initPayload = {
            type: 'file_chunked_v1',
            id: fileId,
            fileName: 'test.txt',
            fileType: 'text/plain',
            fileSize: 20,
            totalChunks: 2,
            bundle: { key_b64: 'k', nonce_b64: 'n' }
        };

        vi.mocked(signalManager.decrypt).mockResolvedValueOnce({ m: JSON.stringify(initPayload) });

        // 1. Receive Init
        await handleIncomingMessage({ sender: 'friend', message: { type: 1, body: 'init' } } as any);

        let state = get(userStore);
        expect(state.chats['friend'].messages[0].attachment?.isChunked).toBe(true);

        // 2. Receive Chunk 1
        const chunk1Payload = { type: 'file_chunk', fileId, index: 0, data: 'Y2h1bmsx' }; // "chunk1" in b64
        vi.mocked(signalManager.decrypt).mockResolvedValueOnce({ m: JSON.stringify(chunk1Payload) });
        vi.mocked(signalManager.decryptMediaChunk).mockResolvedValueOnce(new TextEncoder().encode('chunk1'));

        await handleIncomingMessage({ sender: 'friend', message: { type: 1, body: 'c1' } } as any);

        // 3. Receive Chunk 2
        const chunk2Payload = { type: 'file_chunk', fileId, index: 1, data: 'Y2h1bmsy' }; // "chunk2" in b64
        vi.mocked(signalManager.decrypt).mockResolvedValueOnce({ m: JSON.stringify(chunk2Payload) });
        vi.mocked(signalManager.decryptMediaChunk).mockResolvedValueOnce(new TextEncoder().encode('chunk2'));

        await handleIncomingMessage({ sender: 'friend', message: { type: 1, body: 'c2' } } as any);

        // Verification
        expect(attachmentStore.put).toHaveBeenCalled();
        const call = vi.mocked(attachmentStore.put).mock.calls[0];
        expect(call[0]).toBe(fileId);
        expect(new TextDecoder().decode(call[1])).toBe('chunk1chunk2');

        state = get(userStore);
        const msg = state.chats['friend'].messages[0];
        expect(msg.content).toBe('File: test.txt');
        expect(msg.attachment?.isComplete).toBe(true);
    });
});
