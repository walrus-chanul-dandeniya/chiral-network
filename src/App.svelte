<script lang="ts">
  import './styles/globals.css'
  import { Upload, Download, Shield, Wallet, Menu, Globe, BarChart3, Moon, Sun, Settings, Cpu } from 'lucide-svelte'
  import UploadPage from './pages/Upload.svelte'
  import DownloadPage from './pages/Download.svelte'
  import ProxyPage from './pages/Proxy.svelte'
  import AccountPage from './pages/Account.svelte'
  import NetworkPage from './pages/Network.svelte'
  import AnalyticsPage from './pages/Analytics.svelte'
  import SettingsPage from './pages/Settings.svelte'
  import MiningPage from './pages/Mining.svelte'
  import WindowControls from '$lib/components/WindowControls.svelte'
  import { networkStatus, currentTheme } from '$lib/stores'
  
  let currentPage = 'download'
  let sidebarOpen = true
  
  const menuItems = [
    { id: 'download', label: 'Download', icon: Download },
    { id: 'upload', label: 'Upload', icon: Upload },
    { id: 'network', label: 'Network', icon: Globe },
    { id: 'mining', label: 'Mining', icon: Cpu },
    { id: 'proxy', label: 'Proxy', icon: Shield },
    { id: 'analytics', label: 'Analytics', icon: BarChart3 },
    { id: 'account', label: 'Account', icon: Wallet },
    { id: 'settings', label: 'Settings', icon: Settings },
  ]
  
  function toggleTheme() {
    currentTheme.update(theme => theme === 'dark' ? 'light' : 'dark')
  }
  
  // Apply theme to document
  $: {
    if ($currentTheme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }
</script>

<div class="flex h-screen bg-background">
  <!-- Titlebar - This is the draggable area -->
  <div 
    class="absolute top-0 left-0 right-0 h-10 bg-background/95 backdrop-blur-sm border-b flex items-center px-4 z-50"
    style="-webkit-app-region: drag; app-region: drag;"
  >
    <!-- Window Controls (macOS style) -->
    <WindowControls />
    
    <!-- Menu button -->
    <button
      on:click={() => sidebarOpen = !sidebarOpen}
      class="ml-4 p-1 hover:bg-accent rounded"
      style="-webkit-app-region: no-drag; app-region: no-drag;"
    >
      <Menu class="h-4 w-4" />
    </button>
    
    <!-- App title centered -->
    <div class="absolute left-1/2 transform -translate-x-1/2">
      <span class="text-sm font-medium select-none">Chiral Network</span>
    </div>
    
    <!-- Status and theme toggle on the right -->
    <div class="ml-auto flex items-center gap-3 text-xs select-none">
      <button
        on:click={toggleTheme}
        class="p-1 hover:bg-accent rounded"
        style="-webkit-app-region: no-drag; app-region: no-drag;"
      >
        {#if $currentTheme === 'dark'}
          <Sun class="h-4 w-4" />
        {:else}
          <Moon class="h-4 w-4" />
        {/if}
      </button>
      <div class="flex items-center gap-1">
        <div class="w-2 h-2 rounded-full {$networkStatus === 'connected' ? 'bg-green-500' : 'bg-red-500'}"></div>
        <span class="text-muted-foreground">{$networkStatus}</span>
      </div>
    </div>
  </div>
  
  <!-- Sidebar -->
  {#if sidebarOpen}
    <div class="w-64 bg-card border-r pt-10 transition-all">
      <nav class="p-4 space-y-2">
        {#each menuItems as item}
          <button
            on:click={() => currentPage = item.id}
            class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors {currentPage === item.id ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/50'}"
            style="-webkit-app-region: no-drag; app-region: no-drag;"
          >
            <svelte:component this={item.icon} class="h-4 w-4" />
            {item.label}
          </button>
        {/each}
      </nav>
    </div>
  {/if}
  
  <!-- Main Content -->
  <div class="flex-1 pt-10 overflow-auto">
    <div class="p-6">
      {#if currentPage === 'upload'}
        <UploadPage />
      {:else if currentPage === 'download'}
        <DownloadPage />
      {:else if currentPage === 'network'}
        <NetworkPage />
      {:else if currentPage === 'mining'}
        <MiningPage />
      {:else if currentPage === 'proxy'}
        <ProxyPage />
      {:else if currentPage === 'analytics'}
        <AnalyticsPage />
      {:else if currentPage === 'account'}
        <AccountPage />
      {:else if currentPage === 'settings'}
        <SettingsPage />
      {/if}
    </div>
  </div>
</div>