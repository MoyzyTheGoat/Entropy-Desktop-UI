<script lang="ts">
  import { LucidePaperclip, LucideMic, LucideSend, LucideX } from 'lucide-svelte';
  import type { Message } from '../../lib/types';

  interface Props {
    replyingTo: Message | null;
    onCancelReply: () => void;
    onSend: (text: string) => void;
    onFileSelect: (file: File) => void;
    onToggleRecording: () => void;
    isRecording: boolean;
  }

  let { replyingTo, onCancelReply, onSend, onFileSelect, onToggleRecording, isRecording }: Props = $props();

  let messageInput = $state("");
  let fileInput = $state<HTMLInputElement | null>(null);

  const handleSend = () => {
    if (!messageInput.trim()) return;
    onSend(messageInput);
    messageInput = "";
  };

  const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
    }
  };

  const onFileInputChange = (e: Event) => {
    const files = (e.target as HTMLInputElement).files;
    if (files && files.length > 0) onFileSelect(files[0]);
  };
</script>

<div class="bg-white/95 backdrop-blur-md p-3 pb-4 border-t border-gray-100 z-20">
    {#if replyingTo}
        <div class="mb-3 p-3 bg-indigo-50/80 backdrop-blur rounded-2xl flex items-center justify-between border border-indigo-100 shadow-sm animate-in slide-in-from-bottom-2 duration-200">
            <div class="flex items-center space-x-3 overflow-hidden">
                <div class="w-1 h-8 bg-indigo-500 rounded-full"></div>
                <div class="flex flex-col min-w-0">
                    <span class="text-[10px] font-black uppercase text-indigo-600 tracking-wider">Replying to {replyingTo.senderAlias || 'Peer'}</span>
                    <span class="text-xs text-gray-600 truncate opacity-70 italic">{replyingTo.content}</span>
                </div>
            </div>
            <button onclick={onCancelReply} class="p-2 text-gray-400 hover:text-indigo-600 transition hover:bg-white rounded-xl">
                <LucideX size={18} />
            </button>
        </div>
    {/if}

    <div class="flex items-end space-x-2 max-w-5xl mx-auto">
        <button 
            onclick={() => fileInput?.click()}
            class="p-3.5 text-gray-500 hover:text-indigo-600 hover:bg-indigo-50 rounded-2xl transition active:scale-90"
        >
            <LucidePaperclip size={22} />
        </button>
        <input type="file" bind:this={fileInput} onchange={onFileInputChange} class="hidden" />

        <div class="flex-1 relative group">
            <textarea 
                bind:value={messageInput}
                onkeydown={handleKeydown}
                placeholder="Secure message..."
                class="w-full bg-gray-100 border-none rounded-2xl p-3.5 px-4 text-sm focus:ring-2 focus:ring-indigo-500/20 focus:bg-white transition-all resize-none max-h-32 min-h-[48px] font-medium custom-scrollbar"
                rows="1"
            ></textarea>
        </div>

        {#if messageInput.trim().length > 0}
            <button 
                onclick={handleSend}
                class="p-3.5 bg-indigo-600 text-white rounded-2xl shadow-lg shadow-indigo-200 hover:bg-indigo-700 hover:-translate-y-0.5 transition-all active:scale-95"
            >
                <LucideSend size={22} />
            </button>
        {:else}
            <button 
                onclick={onToggleRecording}
                class="p-3.5 {isRecording ? 'bg-red-500 text-white animate-pulse' : 'bg-gray-100 text-gray-500 hover:bg-indigo-50 hover:text-indigo-600'} rounded-2xl transition-all active:scale-95"
            >
                <LucideMic size={22} />
            </button>
        {/if}
    </div>
</div>

<style>
    textarea::-webkit-scrollbar { width: 4px; }
    textarea::-webkit-scrollbar-track { background: transparent; }
    textarea::-webkit-scrollbar-thumb { background: rgba(0,0,0,0.05); border-radius: 10px; }
</style>
