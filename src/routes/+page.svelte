<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import HostsEditor from "$lib/HostsEditor.svelte";

  // State
  let profiles = $state<string[]>([]);
  let activeProfile = $state<string>("");
  let selectedProfile = $state<string>("");
  let profileContent = $state<string>("");
  let backups = $state<string[]>([]);
  let newProfileName = $state<string>("");
  let errorMessage = $state<string>("");
  let successMessage = $state<string>("");
  let currentView = $state<"editor" | "backup">("editor");
  let showAboutModal = $state<boolean>(false);
  let systemHostsContent = $state<string>("");
  let systemHostsLineCount = $state<number>(0);
  let showCloneModal = $state<boolean>(false);
  let cloneSourceProfile = $state<string>("");
  let showRenameModal = $state<boolean>(false);
  let renameTargetProfile = $state<string>("");
  let renameNewName = $state<string>("");
  
  // Backup viewing state
  let selectedBackup = $state<string>("");
  let backupContent = $state<string>("");
  let backupView = $state<"list" | "view">("list");

  // Load initial data
  onMount(async () => {
    try {
      console.log("Starting initial data load...");
      await loadSystemHosts();
      console.log("System hosts loaded:", systemHostsContent.length, "characters");
      await loadProfiles();
      await loadBackups();
      // Select system hosts by default after loading
      selectedProfile = "live";
      profileContent = systemHostsContent || "# Live System Hosts\n# Loaded content appears here\n";
      console.log("Set initial profile content:", profileContent.length, "characters");
      
      // Force a re-render after a short delay to ensure editor is ready
      setTimeout(() => {
        console.log("Final profile content check:", profileContent.length, "characters");
        // Force reactivity update
        profileContent = profileContent + "";
      }, 200);
    } catch (error) {
      console.error("Failed to load initial data:", error);
      // Set fallback content
      systemHostsContent = "# Failed to load system hosts\n# Please check permissions\n";
      profileContent = systemHostsContent;
      selectedProfile = "live";
    }
    
    // Add keyboard shortcut for Ctrl+S
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 's') {
        e.preventDefault();
        // Only save if we're in editor view and not on live profile
        if (currentView === "editor" && selectedProfile !== "live") {
          saveProfile();
        }
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    
    // Cleanup
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  });

  // Ensure editor updates when content changes
  $effect(() => {
    if (profileContent) {
      console.log("Profile content updated:", profileContent.length, "characters");
    }
  });

  async function loadSystemHosts() {
    try {
      systemHostsContent = await invoke<string>("read_system_hosts");
      systemHostsLineCount = systemHostsContent.split('\n').filter(line => line.trim() && !line.trim().startsWith('#')).length;
    } catch (error) {
      console.error("Failed to load system hosts:", error);
      systemHostsContent = "# Failed to load system hosts\n# Please check permissions\n# This might require administrator rights\n";
      systemHostsLineCount = 0;
      showError(`Failed to load system hosts: ${error}`);
    }
  }

  async function loadProfiles() {
    try {
      profiles = await invoke<string[]>("list_profiles");
      activeProfile = await invoke<string>("get_active_profile");
      if (profiles.length > 0 && !selectedProfile) {
        selectedProfile = activeProfile || profiles[0];
        await loadProfile(selectedProfile);
      }
    } catch (error) {
      showError(`Failed to load profiles: ${error}`);
    }
  }

  async function loadProfile(name: string) {
    try {
      console.log("Loading profile:", name);
      if (name === "live") {
        await loadSystemHosts();
        profileContent = systemHostsContent;
        console.log("Set profile content to system hosts:", profileContent.length, "characters");
        selectedProfile = "live";
      } else {
        profileContent = await invoke<string>("read_profile", { name });
        console.log("Loaded profile content:", profileContent.length, "characters");
        selectedProfile = name;
      }
    } catch (error) {
      console.error("Failed to load profile:", error);
      showError(`Failed to load profile: ${error}`);
    }
  }

  async function saveProfile() {
    if (selectedProfile === "live") {
      showError("Cannot save to live system hosts. Create a profile first.");
      return;
    }
    try {
      await invoke("write_profile", { name: selectedProfile, content: profileContent });
      
      // If this is the active profile, automatically apply the changes
      if (activeProfile === selectedProfile) {
        await invoke("activate_profile", { name: selectedProfile });
        await loadSystemHosts();
        await flushDns();
        showSuccess(`Profile "${selectedProfile}" saved and applied! DNS cache flushed.`);
      } else {
        showSuccess(`Profile "${selectedProfile}" saved successfully!`);
      }
    } catch (error) {
      showError(`Failed to save profile: ${error}`);
    }
  }

  async function createProfile() {
    if (!newProfileName.trim()) {
      showError("Profile name cannot be empty");
      return;
    }
    try {
      await invoke("create_profile", { name: newProfileName });
      await loadProfiles();
      selectedProfile = newProfileName;
      await loadProfile(newProfileName);
      newProfileName = "";
      showSuccess("Profile created successfully!");
      // Close modal
      (document.getElementById('create_profile_modal') as any)?.close();
    } catch (error) {
      showError(`Failed to create profile: ${error}`);
    }
  }

  async function deleteProfile(name: string) {
    if (!confirm(`Are you sure you want to delete profile "${name}"?`)) {
      return;
    }
    try {
      await invoke("delete_profile", { name });
      await loadProfiles();
      if (selectedProfile === name) {
        selectedProfile = "current";
        await loadProfile("current");
      }
      showSuccess("Profile deleted successfully!");
    } catch (error) {
      showError(`Failed to delete profile: ${error}`);
    }
  }

  function openCloneModal(sourceName: string) {
    cloneSourceProfile = sourceName;
    newProfileName = sourceName === "live" ? "" : `${sourceName}_copy`;
    showCloneModal = true;
  }

  async function cloneProfile() {
    if (!newProfileName.trim()) {
      showError("Profile name cannot be empty");
      return;
    }
    try {
      // Create new profile
      await invoke("create_profile", { name: newProfileName });
      
      // Get content from source
      let content = "";
      if (cloneSourceProfile === "live") {
        content = systemHostsContent;
      } else {
        content = await invoke<string>("read_profile", { name: cloneSourceProfile });
      }
      
      // Write content to new profile
      await invoke("write_profile", { name: newProfileName, content });
      
      await loadProfiles();
      selectedProfile = newProfileName;
      await loadProfile(newProfileName);
      showSuccess(`Profile "${newProfileName}" cloned successfully!`);
      showCloneModal = false;
      newProfileName = "";
    } catch (error) {
      showError(`Failed to clone profile: ${error}`);
    }
  }

  function openRenameModal(profileName: string) {
    renameTargetProfile = profileName;
    renameNewName = profileName;
    showRenameModal = true;
  }

  async function renameProfile() {
    if (!renameNewName.trim()) {
      showError("Profile name cannot be empty");
      return;
    }
    if (renameNewName === renameTargetProfile) {
      showRenameModal = false;
      return;
    }
    try {
      await invoke("rename_profile", { oldName: renameTargetProfile, newName: renameNewName });
      await loadProfiles();
      if (selectedProfile === renameTargetProfile) {
        selectedProfile = renameNewName;
      }
      showSuccess(`Profile renamed to "${renameNewName}"!`);
      showRenameModal = false;
    } catch (error) {
      showError(`Failed to rename profile: ${error}`);
    }
  }

  async function activateProfile(name: string) {
    if (!confirm(`Activate profile "${name}"? This will modify your system hosts file.`)) {
      return;
    }
    try {
      await invoke("activate_profile", { name });
      activeProfile = name;
      showSuccess(`Profile "${name}" activated! DNS cache flushed.`);
    } catch (error) {
      showError(`Failed to activate profile: ${error}`);
      // Don't change activeProfile - the toggle will stay in correct state
    }
  }

  async function loadBackups() {
    try {
      backups = await invoke<string[]>("list_backups");
    } catch (error) {
      showError(`Failed to load backups: ${error}`);
    }
  }

  async function loadBackupContent(backupName: string) {
    try {
      backupContent = await invoke<string>("read_backup", { backupName });
      selectedBackup = backupName;
      backupView = "view";
    } catch (error) {
      showError(`Failed to load backup content: ${error}`);
    }
  }

  function backToBackupList() {
    backupView = "list";
    selectedBackup = "";
    backupContent = "";
  }

  async function createBackup() {
    try {
      const backupPath = await invoke<string>("backup_hosts");
      await loadBackups();
      showSuccess(`Backup created: ${backupPath}`);
    } catch (error) {
      showError(`Failed to create backup: ${error}`);
    }
  }

  async function deleteBackup(backupName: string) {
    if (!confirm(`Are you sure you want to delete backup "${backupName}"?`)) {
      return;
    }
    try {
      await invoke("delete_backup", { backupName });
      await loadBackups();
      if (selectedBackup === backupName) {
        backToBackupList();
      }
      showSuccess(`Backup deleted successfully`);
    } catch (error) {
      showError(`Failed to delete backup: ${error}`);
    }
  }

  async function deleteAllBackups() {
    if (!confirm(`Are you sure you want to delete ALL ${backups.length} backups? This cannot be undone!`)) {
      return;
    }
    try {
      await invoke("delete_all_backups");
      await loadBackups();
      backToBackupList();
      showSuccess(`All backups deleted successfully`);
    } catch (error) {
      showError(`Failed to delete backups: ${error}`);
    }
  }

  async function restoreBackup(backupName: string) {
    // Extract date from backup filename (e.g., hosts_backup_20251030_162552.txt -> 20251030_162552)
    const dateMatch = backupName.match(/(\d{8}_\d{6})/);
    const dateStr = dateMatch ? dateMatch[1] : new Date().toISOString().replace(/[-:]/g, '').split('.')[0];
    const profileName = `restore_${dateStr}`;
    
    if (!confirm(`Restore backup "${backupName}"?\n\nThis will create a new profile "${profileName}" and activate it.`)) {
      return;
    }
    
    try {
      // Read backup content
      const backupContent = await invoke<string>("read_backup", { backupName });
      
      // Create new profile with backup content
      await invoke("create_profile", { name: profileName });
      await invoke("write_profile", { name: profileName, content: backupContent });
      
      // Activate the new profile
      await invoke("activate_profile", { name: profileName });
      
      // Reload everything
      await loadSystemHosts();
      await loadProfiles();
      await loadBackups();
      
      // Switch to profiles view and select the new profile
      currentView = "editor";
      selectedProfile = profileName;
      profileContent = backupContent;
      
      showSuccess(`Backup restored as profile "${profileName}" and activated!`);
    } catch (error) {
      showError(`Failed to restore backup: ${error}`);
    }
  }

  async function flushDns() {
    try {
      await invoke("flush_dns");
      showSuccess("DNS cache flushed successfully!");
    } catch (error) {
      showError(`Failed to flush DNS: ${error}`);
    }
  }

  function showError(message: string) {
    errorMessage = message;
    setTimeout(() => errorMessage = "", 5000);
  }

  function showSuccess(message: string) {
    successMessage = message;
    setTimeout(() => successMessage = "", 3000);
  }
</script>

<div class="flex h-screen bg-gradient-to-r from-base-100 to-base-200">
  <!-- Sidebar -->
  <div class="w-64 bg-base-100 shadow-xl flex flex-col">
    <div class="p-4 border-b border-base-300">
      <h1 class="text-2xl font-bold text-primary">HostsPilot</h1>
      <p class="text-xs text-base-content/60">Manage your hosts files</p>
      <p class="text-xs text-base-content/40 mt-1">by Vladimir Strycek</p>
    </div>

    <!-- Navigation Tabs -->
    <div class="px-4 pb-4">
      <div class="flex gap-2">
        <button 
          type="button"
          class="flex-1 btn nav-tab-btn {currentView === 'editor' ? 'btn-primary nav-tab-active' : 'btn-outline'}"
          onclick={() => currentView = "editor"}
          tabindex="0"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
          Profiles
        </button>
        <button 
          type="button"
          class="flex-1 btn nav-tab-btn {currentView === 'backup' ? 'btn-primary nav-tab-active' : 'btn-outline'}"
          onclick={() => currentView = "backup"}
          tabindex="0"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3-3m0 0l-3 3m3-3v12" />
          </svg>
          Backups
        </button>
      </div>
    </div>

    {#if currentView === "editor"}
      <!-- Profiles List -->
      <div class="flex-1 overflow-y-auto p-4 flex flex-col">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-lg font-semibold">Hosts Profiles</h2>
          <div class="badge badge-outline badge-sm">
            {profiles.length + 1} total
          </div>
        </div>

        <div class="space-y-2 flex-1">
          <!-- Live System Hosts -->
          <div 
            class="card live-card shadow-sm hover:shadow-lg transition-all cursor-pointer {selectedProfile === 'live' ? 'ring-2 ring-primary' : ''}"
            role="button"
            tabindex="0"
            onclick={() => loadProfile('live')}
            onkeydown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                loadProfile('live');
              }
            }}
            aria-label="View live system hosts file"
          >
            <div class="card-body p-3">
              <div class="flex items-center justify-between mb-2">
                <div class="flex-1">
                  <div class="font-bold text-lg {selectedProfile === 'live' ? 'text-primary' : 'text-base-content'}">
                    Live
                    <span class="text-xs text-base-content/60 ml-2 font-normal">({systemHostsLineCount} entries)</span>
                  </div>
                </div>
              </div>
              <div class="flex gap-1">
                <button class="btn btn-xs btn-ghost" onclick={(e) => { e.stopPropagation(); openCloneModal('live'); }} title="Clone">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  Clone
                </button>
              </div>
            </div>
          </div>

          <!-- User Profiles -->
          {#each profiles as profile}
            <div 
              class="card bg-base-200 shadow-sm hover:bg-base-300 transition-colors cursor-pointer {selectedProfile === profile ? 'ring-2 ring-primary' : ''}"
              role="button"
              tabindex="0"
              onclick={() => loadProfile(profile)}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  loadProfile(profile);
                }
              }}
              aria-label={`Select profile ${profile}`}
            >
              <div class="card-body p-3">
                <div class="flex items-center justify-between mb-2">
                  <div class="flex-1">
                    <div class="font-medium {selectedProfile === profile ? 'text-primary' : ''}">
                      {profile}
                    </div>
                  </div>
                  <input 
                    type="checkbox" 
                    class="toggle toggle-sm {activeProfile === profile ? 'toggle-success' : 'toggle-error'}" 
                    checked={activeProfile === profile}
                    onchange={(e) => {
                      if (e.target.checked && activeProfile !== profile) {
                        // Only activate if currently inactive and trying to activate
                        activateProfile(profile);
                      } else if (!e.target.checked && activeProfile === profile) {
                        // Don't allow deactivation through toggle - profiles should always be active
                        e.target.checked = true;
                      }
                    }}
                    onclick={(e) => e.stopPropagation()}
                  />
                </div>
                <div class="flex gap-1">
                  <button class="btn btn-xs btn-ghost" onclick={(e) => { e.stopPropagation(); openRenameModal(profile); }} title="Rename">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                    Rename
                  </button>
                  <button class="btn btn-xs btn-ghost" onclick={(e) => { e.stopPropagation(); openCloneModal(profile); }} title="Clone">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                    Clone
                  </button>
                  <button 
                    class="btn btn-xs btn-ghost text-error" 
                    onclick={(e) => { e.stopPropagation(); deleteProfile(profile); }} 
                    disabled={activeProfile === profile}
                    title="Delete"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                    Delete
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>

        <!-- Add New Profile Button -->
        <div class="mt-4 pt-4 border-t border-base-300">
          <button 
            class="btn btn-primary btn-block btn-sm new-profile-btn"
            onclick={() => (document.getElementById('create_profile_modal') as any)?.showModal()}
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            New Blank Profile
          </button>
        </div>
      </div>
    {:else}
      <!-- Backups List -->
      <div class="flex-1 overflow-y-auto p-4">
        <div class="mb-4">
          <div class="flex justify-between items-center mb-2">
            <h2 class="text-lg font-semibold">Backup History</h2>
            <div class="badge badge-outline badge-sm">
              {backups.length} {backups.length === 1 ? 'backup' : 'backups'}
            </div>
          </div>
          <button class="btn btn-sm btn-primary btn-block create-backup-btn" onclick={createBackup}>
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Create Backup
          </button>
        </div>

        {#if backups.length === 0}
          <div class="text-center py-8 text-base-content/60">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mx-auto mb-4 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p>No backups found</p>
            <p class="text-sm mt-2">Create a backup to get started</p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each backups as backup}
              <div 
                class="card bg-base-200 shadow-sm hover:bg-base-300 transition-colors cursor-pointer {selectedBackup === backup ? 'ring-2 ring-primary' : ''}" 
                role="button"
                tabindex="0"
                onclick={() => loadBackupContent(backup)}
                onkeydown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    loadBackupContent(backup);
                  }
                }}
                aria-label={`View backup ${backup}`}
              >
                <div class="card-body p-3">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-primary flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                        <span class="text-sm font-medium truncate" title={backup}>{backup.replace('hosts_backup_', '').replace('.txt', '')}</span>
                      </div>
                      <p class="text-xs text-base-content/60 mt-1">Click to view</p>
                    </div>
                    <button 
                      class="btn btn-xs btn-ghost btn-circle text-error"
                      onclick={(e) => {
                        e.stopPropagation();
                        deleteBackup(backup);
                      }}
                      title="Delete this backup"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
          
          <!-- Delete All Button at Bottom -->
          <div class="mt-4 pt-4 border-t border-base-300">
            <button class="btn btn-sm btn-error btn-block delete-all-btn" onclick={deleteAllBackups}>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
              Delete All Backups ({backups.length})
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Main Content -->
  <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
    <!-- Navbar -->
    <div class="navbar bg-base-100 shadow-lg flex-shrink-0">
      <div class="flex-1">
        <span class="text-xl font-semibold">
          {#if currentView === "editor"}
            {selectedProfile === "live" ? "Live System Hosts" : (selectedProfile || "No Profile Selected")}
          {:else if backupView === "view"}
            Backup: {selectedBackup}
          {:else}
            Backup & Restore
          {/if}
        </span>
      </div>
      <div class="flex-none gap-2">
        <button class="btn btn-sm btn-ghost flush-dns-btn" onclick={flushDns}>
          Flush DNS
        </button>
        <button 
          class="btn btn-ghost btn-circle" 
          onclick={() => showAboutModal = true}
          aria-label="About"
          title="About HostsPilot"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="w-5 h-5 stroke-current">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
        </button>
      </div>
    </div>

    <!-- Alerts -->
    <div class="flex-shrink-0">
      {#if errorMessage}
        <div class="alert alert-error m-4">
          <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{errorMessage}</span>
        </div>
      {/if}

      {#if successMessage}
        <div class="alert alert-success m-4">
          <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{successMessage}</span>
        </div>
      {/if}
    </div>

    <!-- Editor Content -->
    {#if currentView === "editor"}
      <div class="flex-1 flex flex-col min-h-0 overflow-hidden">
        {#if selectedProfile && profileContent}
          <div class="flex-1 min-h-0 m-0 rounded-none border-0 border-b overflow-hidden">
            {#key selectedProfile}
              <HostsEditor 
                value={profileContent}
                readonly={selectedProfile === "live"}
                onchange={(newContent) => {
                  if (selectedProfile !== "live") {
                    profileContent = newContent;
                  }
                }}
              />
            {/key}
          </div>
          <div class="flex gap-2 p-4 flex-shrink-0 bg-base-100">
            {#if selectedProfile !== "live"}
              <button class="btn btn-sm btn-primary save-profile-btn" onclick={saveProfile}>
                {activeProfile === selectedProfile ? 'Save & Apply' : 'Save Profile'}
              </button>
            {:else}
              <div class="alert alert-info live-hosts-alert">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-5 h-5">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
                <span class="text-sm">This is your live system hosts file (read-only). Clone it to create a new profile.</span>
              </div>
            {/if}
          </div>
        {:else if selectedProfile && !profileContent}
          <div class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <div class="loading loading-spinner loading-lg text-primary"></div>
              <p class="text-lg text-base-content/60 mt-4">Loading profile content...</p>
            </div>
          </div>
        {:else}
          <div class="flex-1 flex items-center justify-center">
            <div class="text-center">
              <p class="text-lg text-base-content/60">No profile selected</p>
              <p class="text-sm text-base-content/40">Create or select a profile to get started</p>
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex-1 flex flex-col min-h-0 overflow-hidden">
        {#if selectedBackup}
          <!-- Backup View -->
          <div class="bg-base-100 p-4 border-b border-base-300 flex-shrink-0">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div>
                  <h3 class="font-semibold text-sm text-base-content/60">Viewing Backup (read-only)</h3>
                  <p class="text-xs text-base-content/50 mt-1">{selectedBackup}</p>
                </div>
              </div>
              <div class="flex gap-2">
                <button 
                  class="btn btn-sm btn-ghost"
                  onclick={backToBackupList}
                  title="Back to backup list"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                  </svg>
                  Back to List
                </button>
                <button 
                  class="btn btn-sm btn-success restore-backup-btn"
                  onclick={() => restoreBackup(selectedBackup)}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                  Restore Backup
                </button>
              </div>
            </div>
          </div>

          <!-- Backup Content Editor -->
          <div class="flex-1 min-h-0 m-0 rounded-none border-0 border-b overflow-hidden">
            <HostsEditor 
              value={backupContent}
              readonly={true}
            />
          </div>

          <!-- Backup Info -->
          <div class="bg-base-100 p-4 border-t border-base-300 flex-shrink-0">
            <div class="alert alert-info">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
              <div>
                <p class="font-medium">Read-only Backup View</p>
                <p class="text-sm">This is a read-only view of the backup file. Click "Restore Backup" to apply it to your system hosts file.</p>
              </div>
            </div>
          </div>
        {:else}
          <!-- Backup Information -->
          <div class="flex-1 flex flex-col min-h-0 overflow-hidden">
            <div class="flex-1 flex items-center justify-center p-4">
              <div class="text-center max-w-md">
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <h2 class="card-title justify-center">
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                      </svg>
                      Backup & Restore
                    </h2>
                    <p class="text-base-content/60 mb-4">
                      Backups are automatically created when you activate a profile or restore from a backup. You can also manually create a backup of your current hosts file.
                    </p>
                    <div class="divider">Instructions</div>
                    <div class="text-left space-y-2">
                      <div class="flex items-start gap-3">
                        <span class="badge badge-primary">1</span>
                        <p class="text-sm">Select a backup from the list on the left to view its contents</p>
                      </div>
                      <div class="flex items-start gap-3">
                        <span class="badge badge-primary">2</span>
                        <p class="text-sm">Review the backup contents in the editor</p>
                      </div>
                      <div class="flex items-start gap-3">
                        <span class="badge badge-primary">3</span>
                        <p class="text-sm">Click "Restore Backup" to apply it to your system</p>
                      </div>
                    </div>
                    <div class="divider">Storage</div>
                    <p class="text-sm text-base-content/60">
                      Backups are stored in: <code class="bg-base-200 px-2 py-1 rounded">%AppData%\HostsPilot\backups</code>
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<!-- Create Profile Modal -->
<dialog id="create_profile_modal" class="modal">
  <div class="modal-box">
    <h3 class="font-bold text-lg">Create New Profile</h3>
    <div class="py-4">
      <input
        type="text"
        placeholder="Profile name"
        class="input input-bordered w-full"
        bind:value={newProfileName}
        onkeydown={(e) => e.key === 'Enter' && createProfile()}
      />
    </div>
    <div class="modal-action">
      <form method="dialog">
        <button class="btn btn-sm modal-cancel-btn">Cancel</button>
      </form>
      <button class="btn btn-sm btn-primary modal-create-btn" onclick={createProfile}>Create</button>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button>close</button>
  </form>
</dialog>

<!-- Clone Profile Modal -->
{#if showCloneModal}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Clone Profile</h3>
      <p class="text-sm text-base-content/60 mt-2">
        Cloning from: <span class="font-semibold">{cloneSourceProfile}</span>
      </p>
      <div class="py-4">
        <input
          type="text"
          placeholder="New profile name"
          class="input input-bordered w-full"
          bind:value={newProfileName}
          onkeydown={(e) => e.key === 'Enter' && cloneProfile()}
        />
      </div>
      <div class="modal-action">
        <button class="btn btn-sm modal-cancel-btn" onclick={() => showCloneModal = false}>Cancel</button>
        <button class="btn btn-sm btn-primary modal-create-btn" onclick={cloneProfile}>Clone</button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={() => showCloneModal = false} onkeydown={(e) => e.key === 'Enter' && (showCloneModal = false)} role="button" tabindex="0" aria-label="Close modal"></div>
  </div>
{/if}

<!-- Rename Profile Modal -->
{#if showRenameModal}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Rename Profile</h3>
      <p class="text-sm text-base-content/60 mt-2">
        Renaming: <span class="font-semibold">{renameTargetProfile}</span>
      </p>
      <div class="py-4">
        <input
          type="text"
          placeholder="New profile name"
          class="input input-bordered w-full"
          bind:value={renameNewName}
          onkeydown={(e) => e.key === 'Enter' && renameProfile()}
        />
      </div>
      <div class="modal-action">
        <button class="btn btn-sm modal-cancel-btn" onclick={() => showRenameModal = false}>Cancel</button>
        <button class="btn btn-sm btn-primary modal-create-btn" onclick={renameProfile}>Rename</button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={() => showRenameModal = false} onkeydown={(e) => e.key === 'Enter' && (showRenameModal = false)} role="button" tabindex="0" aria-label="Close modal"></div>
  </div>
{/if}

<!-- About Modal -->
{#if showAboutModal}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-2xl mb-4 text-primary">HostsPilot</h3>
      <div class="space-y-3">
        <div>
          <p class="text-sm text-base-content/60">Version</p>
          <p class="font-semibold">1.0.0</p>
        </div>
        <div>
          <p class="text-sm text-base-content/60">Author</p>
          <p class="font-semibold">Vladimir Strycek</p>
        </div>
        <div>
          <p class="text-sm text-base-content/60">Repository</p>
          <a 
            href="https://github.com/VladoPortos/hostspilot" 
            target="_blank"
            class="link link-primary font-semibold"
          >
            github.com/VladoPortos/hostspilot
          </a>
        </div>
        <div class="divider"></div>
        <p class="text-sm text-base-content/70">
          A lightweight Windows desktop app for managing and switching between multiple hosts file profiles.
        </p>
        <p class="text-xs text-base-content/50">
          Built with Rust, Tauri 2, Svelte 5, and TailwindCSS
        </p>
      </div>
      <div class="modal-action">
        <button class="btn btn-sm btn-primary modal-create-btn" onclick={() => showAboutModal = false}>Close</button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={() => showAboutModal = false} onkeydown={(e) => e.key === 'Enter' && (showAboutModal = false)} role="button" tabindex="0" aria-label="Close modal"></div>
  </div>
{/if}

<style>
  /* Enhanced toggle colors - green for active, red for inactive */
  :global(.toggle-success:checked) {
    --tw-bg-opacity: 1;
    background-color: rgb(34 197 94 / var(--tw-bg-opacity)) !important;
    border-color: rgb(34 197 94 / var(--tw-bg-opacity)) !important;
  }
  
  :global(.toggle-error:not(:checked)) {
    --tw-bg-opacity: 1;
    background-color: rgb(239 68 68 / var(--tw-bg-opacity)) !important;
    border-color: rgb(239 68 68 / var(--tw-bg-opacity)) !important;
  }
  
  /* Animated diagonal stripes for Live card */
  .live-card {
    background: repeating-linear-gradient(
      45deg,
      rgba(34, 197, 94, 0.15),
      rgba(34, 197, 94, 0.15) 10px,
      rgba(255, 255, 255, 0.05) 10px,
      rgba(255, 255, 255, 0.05) 20px
    );
    background-size: 28px 28px;
    animation: stripeMove 1s linear infinite;
    border: 1px solid rgba(34, 197, 94, 0.4);
    position: relative;
    overflow: hidden;
  }
  
  @keyframes stripeMove {
    0% {
      background-position: 0 0;
    }
    100% {
      background-position: 28px 28px;
    }
  }
  
  .live-card:hover {
    border-color: rgba(34, 197, 94, 0.6);
    box-shadow: 0 4px 12px rgba(34, 197, 94, 0.25);
  }
  
  /* Delete All Backups button - custom styling with visible border */
  .delete-all-btn {
    border: 2px solid rgba(127, 29, 29, 1) !important;
  }
  
  .delete-all-btn:hover {
    border-color: rgba(127, 29, 29, 1) !important;
    background-color: rgba(185, 28, 28, 1) !important;
    color: white !important;
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }
  
  /* Create Backup button - add visible darker blue border */
  .create-backup-btn {
    border: 2px solid rgba(29, 78, 216, 1) !important;
  }
  
  .create-backup-btn:hover {
    border-color: rgba(29, 78, 216, 1) !important;
    background-color: rgba(59, 130, 246, 1) !important;
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }
  
  /* New Blank Profile button - black outline only */
  .new-profile-btn {
    border: 1px solid rgba(0, 0, 0, 0.3) !important;
  }
  
  /* Save Profile / Save & Apply button - black outline only */
  .save-profile-btn {
    border: 1px solid rgba(0, 0, 0, 0.3) !important;
    padding-left: 1rem !important;
    padding-right: 1rem !important;
  }
  
  /* Flush DNS button - thin border and hover effect */
  .flush-dns-btn {
    border: 1px solid rgba(0, 0, 0, 0.3) !important;
    padding-left: 1rem !important;
    padding-right: 1rem !important;
  }
  
  .flush-dns-btn:hover {
    background-color: rgba(238, 9, 9, 0.6) !important;
    color: white !important;
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }
  
  /* Navigation tab buttons - custom colored background when active */
  .nav-tab-btn {
    padding-top: 0.75rem !important;
  }
  
  .nav-tab-active {
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.9), rgba(37, 99, 235, 0.9)) !important;
    border: 1px solid rgba(29, 78, 216, 0.8) !important;
    color: white !important;
    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3) !important;
  }
  
  /* Live hosts alert - reduce height by 20% */
  .live-hosts-alert {
    padding-top: 0.3rem !important;
    padding-bottom: 0.3rem !important;
    min-height: auto !important;
  }
  
  /* Restore Backup button - green border and hover effect */
  .restore-backup-btn {
    border: 2px solid rgba(22, 163, 74, 1) !important;
    padding-left: 1rem !important;
    padding-right: 1rem !important;
  }
  
  .restore-backup-btn:hover {
    border-color: rgba(22, 163, 74, 0.3) !important;
    background-color: rgba(34, 197, 94, 0.3) !important;
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }
  
  /* Modal buttons - match New Blank Profile button style */
  .modal-cancel-btn {
    border: 1px solid rgba(0, 0, 0, 0.3) !important;
    padding-left: 1rem !important;
    padding-right: 1rem !important;
  }
  
  .modal-create-btn {
    border: 1px solid rgba(0, 0, 0, 0.3) !important;
    padding-left: 1rem !important;
    padding-right: 1rem !important;
  }
  
  /* Modal input boxes - slightly different background with thin border */
  .modal-box .input {
    background-color: rgba(255, 255, 255, 0.05) !important;
    border: 1px solid rgba(0, 0, 0, 0.3) !important;
    padding-left: 1rem !important;
    padding-right: 1rem !important;
  }
  
  .modal-box .input:focus {
    background-color: rgba(255, 255, 255, 0.08) !important;
    border-color: oklch(var(--p)) !important;
    outline: none !important;
  }
</style>
