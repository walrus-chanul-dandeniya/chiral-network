<script lang="ts">
  // Only show if we're in Tauri
  let isTauri = false
  let appWindow: any = null
  
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    isTauri = true
    import('@tauri-apps/api/window').then(module => {
      appWindow = module.appWindow
    })
  }
  
  async function minimizeWindow() {
    if (appWindow) await appWindow.minimize()
  }
  
  async function maximizeWindow() {
    if (appWindow) await appWindow.toggleMaximize()
  }
  
  async function closeWindow() {
    if (appWindow) await appWindow.close()
  }
</script>

{#if isTauri}
  <div class="flex items-center gap-2" style="-webkit-app-region: no-drag; app-region: no-drag;">
    <button
      on:click={minimizeWindow}
      class="w-3 h-3 rounded-full bg-yellow-500 hover:bg-yellow-600 transition-colors"
      aria-label="Minimize"
    />
    <button
      on:click={maximizeWindow}
      class="w-3 h-3 rounded-full bg-green-500 hover:bg-green-600 transition-colors"
      aria-label="Maximize"
    />
    <button
      on:click={closeWindow}
      class="w-3 h-3 rounded-full bg-red-500 hover:bg-red-600 transition-colors"
      aria-label="Close"
    />
  </div>
{/if}