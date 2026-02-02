<script lang="ts">
  import { LucidePhone, LucideVideo, LucideSearch, LucideMoreVertical, LucideShieldCheck, LucideShieldAlert } from 'lucide-svelte';
  import type { Chat } from '../../lib/types';

  interface Props {
    activeChat: Chat;
    onInitiateCall: (type: 'voice' | 'video') => void;
    onToggleGallery: () => void;
    onToggleSearch: () => void;
    onShowOptions: () => void;
  }

  let { activeChat, onInitiateCall, onToggleGallery, onToggleSearch, onShowOptions }: Props = $props();
</script>

<div class="bg-white/95 backdrop-blur-md p-3 px-4 border-b border-gray-200 flex justify-between items-center shadow-sm z-30">
    <div 
        class="flex items-center space-x-3 overflow-hidden cursor-pointer group" 
        onclick={onToggleGallery}
    >
        <div class="w-10 h-10 rounded-xl bg-gradient-to-tr {activeChat.isGroup ? 'from-purple-500 to-indigo-600' : 'from-blue-400 to-blue-600'} shrink-0 flex items-center justify-center text-white font-bold shadow-sm relative overflow-hidden transition group-hover:scale-105">
            {#if activeChat.pfp}
                <img src={activeChat.pfp} alt="" class="w-full h-full object-cover" />
            {:else}
                <span class="text-lg">{(activeChat.localNickname || activeChat.peerAlias || "?")[0].toUpperCase()}</span>
            {/if}
            {#if activeChat.isOnline}
                <div class="absolute bottom-0 right-0 w-3 h-3 bg-emerald-500 border-2 border-white rounded-full"></div>
            {/if}
        </div>
        <div class="flex flex-col min-w-0">
            <h3 class="font-black text-sm text-gray-900 truncate tracking-tight flex items-center space-x-1.5">
                <span>{activeChat.localNickname || activeChat.peerAlias}</span>
                {#if activeChat.isVerified}
                    <LucideShieldCheck size={14} class="text-emerald-500" />
                {:else if !activeChat.isGroup}
                    <LucideShieldAlert size={14} class="text-amber-500 opacity-50" />
                {/if}
            </h3>
            <span class="text-[10px] font-bold uppercase tracking-widest {activeChat.isOnline ? 'text-emerald-500' : 'text-gray-400'}">
                {activeChat.isOnline ? 'OnlineNow' : activeChat.isTyping ? 'Typing...' : 'EncryptedSignal'}
            </span>
        </div>
    </div>

    <div class="flex items-center space-x-1">
        {#if !activeChat.isGroup}
            <button onclick={() => onInitiateCall('voice')} class="p-2.5 text-gray-500 hover:text-indigo-600 hover:bg-gray-100 rounded-xl transition active:scale-90"><LucidePhone size={20} /></button>
            <button onclick={() => onInitiateCall('video')} class="p-2.5 text-gray-500 hover:text-indigo-600 hover:bg-gray-100 rounded-xl transition active:scale-90"><LucideVideo size={20} /></button>
        {/if}
        <div class="w-px h-6 bg-gray-200 mx-1"></div>
        <button onclick={onToggleSearch} class="p-2.5 text-gray-500 hover:text-indigo-600 hover:bg-gray-100 rounded-xl transition active:scale-90"><LucideSearch size={20} /></button>
        <button onclick={onShowOptions} class="p-2.5 text-gray-500 hover:text-indigo-600 hover:bg-gray-100 rounded-xl transition active:scale-90"><LucideMoreVertical size={20} /></button>
    </div>
</div>
