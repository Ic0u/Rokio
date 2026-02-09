<script lang="ts">
  import { Eye, EyeOff, Loader2 } from "lucide-svelte";
  import { auth } from "$lib/stores";
  import { onMount } from "svelte";

  let { vaultExists }: { vaultExists: boolean } = $props();

  let password = $state("");
  let confirmPassword = $state("");
  let showPassword = $state(false);
  let loading = $state(false);
  let error = $state("");
  let mounted = $state(false);

  onMount(() => {
    setTimeout(() => mounted = true, 50);
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = "";

    if (!password) {
      error = "Password is required";
      return;
    }

    if (!vaultExists) {
      if (password.length < 6) {
        error = "Password must be at least 6 characters";
        return;
      }
      if (password !== confirmPassword) {
        error = "Passwords do not match";
        return;
      }
    }

    loading = true;

    try {
      if (vaultExists) {
        const success = await auth.unlock(password);
        if (!success) {
          error = "Incorrect password";
        }
      } else {
        await auth.createVault(password);
      }
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="lock-screen">
  <div class="lock-container" class:mounted>
    <!-- Cute Mascot with enhanced animation -->
    <div class="mascot-container">
      <div class="mascot-glow"></div>
      <img src="/cute.png" alt="ROKIO" class="mascot" />
    </div>

    <h1 class="app-title">Rokio</h1>
    <p class="app-subtitle">
      {vaultExists ? "Enter password to unlock" : "Create master password"}
    </p>

    <form onsubmit={handleSubmit} class="lock-form">
      <div class="input-group">
        <input
          type={showPassword ? "text" : "password"}
          bind:value={password}
          placeholder={vaultExists ? "Password" : "Create password"}
          autocomplete="off"
        />
        <button
          type="button"
          class="toggle-password"
          onclick={() => (showPassword = !showPassword)}
        >
          {#if showPassword}
            <EyeOff size={16} />
          {:else}
            <Eye size={16} />
          {/if}
        </button>
      </div>

      {#if !vaultExists}
        <div class="input-group">
          <input
            type={showPassword ? "text" : "password"}
            bind:value={confirmPassword}
            placeholder="Confirm password"
            autocomplete="off"
          />
        </div>
      {/if}

      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <button type="submit" class="unlock-btn" disabled={loading}>
        {#if loading}
          <Loader2 size={18} class="spin" />
        {:else}
          {vaultExists ? "Unlock" : "Create Vault"}
        {/if}
      </button>
    </form>

    {#if !vaultExists}
      <p class="security-note">
        AES-256-GCM encryption with hardware-bound keys
      </p>
    {/if}
  </div>
</div>

<style>
  /* Import Google Font - Playwrite for elegant italic */
  @import url('https://fonts.googleapis.com/css2?family=Instrument+Serif:ital@1&display=swap');

  .lock-screen {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1c1c1e;
    overflow: hidden;
  }

  .lock-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 280px;
    padding: 32px 24px;
    text-align: center;
    opacity: 0;
    transform: translateY(24px) scale(0.96);
    transition: all 0.6s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .lock-container.mounted {
    opacity: 1;
    transform: translateY(0) scale(1);
  }

  /* Mascot with glow effect */
  .mascot-container {
    position: relative;
    margin-bottom: 20px;
  }

  .mascot-glow {
    position: absolute;
    inset: 0;
    background: radial-gradient(circle, rgba(197, 61, 61, 0.3) 0%, transparent 70%);
    filter: blur(20px);
    animation: pulse-glow 3s ease-in-out infinite;
  }

  @keyframes pulse-glow {
    0%, 100% { opacity: 0.5; transform: scale(1); }
    50% { opacity: 0.8; transform: scale(1.1); }
  }

  .mascot {
    position: relative;
    width: 110px;
    height: auto;
    animation: float 4s ease-in-out infinite;
    filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.3));
  }

  @keyframes float {
    0%, 100% { 
      transform: translateY(0) rotate(-2deg); 
    }
    25% {
      transform: translateY(-6px) rotate(0deg);
    }
    50% { 
      transform: translateY(-10px) rotate(2deg); 
    }
    75% {
      transform: translateY(-6px) rotate(0deg);
    }
  }

  /* Elegant italic title with serif font */
  .app-title {
    font-family: 'Instrument Serif', Georgia, serif;
    font-size: 36px;
    font-weight: 400;
    font-style: italic;
    margin: 0 0 6px 0;
    letter-spacing: 1px;
    color: #fff;
    animation: title-fade 0.8s ease-out 0.3s both;
  }

  @keyframes title-fade {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .app-subtitle {
    font-size: 13px;
    color: #8e8e93;
    margin: 0 0 28px 0;
    font-weight: 400;
    animation: subtitle-fade 0.8s ease-out 0.5s both;
  }

  @keyframes subtitle-fade {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .lock-form {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    animation: form-fade 0.8s ease-out 0.6s both;
  }

  @keyframes form-fade {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .input-group {
    position: relative;
    width: 100%;
  }

  input {
    width: 100%;
    padding: 12px 40px 12px 14px;
    background: #2c2c2e;
    border: none;
    border-radius: 10px;
    color: #fff;
    font-size: 15px;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }

  input:focus {
    outline: none;
    background: #3a3a3c;
    box-shadow: 0 0 0 2px rgba(197, 61, 61, 0.3);
  }

  input::placeholder {
    color: #636366;
  }

  .toggle-password {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: #636366;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.15s;
  }

  .toggle-password:hover {
    color: #8e8e93;
  }

  .error-message {
    padding: 10px 12px;
    background: rgba(255, 69, 58, 0.12);
    border-radius: 8px;
    color: #ff453a;
    font-size: 13px;
    animation: shake 0.4s ease-out;
  }

  @keyframes shake {
    0%, 100% { transform: translateX(0); }
    20% { transform: translateX(-6px); }
    40% { transform: translateX(6px); }
    60% { transform: translateX(-4px); }
    80% { transform: translateX(4px); }
  }

  .unlock-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 100%;
    padding: 12px;
    background: #c53d3d;
    border: none;
    border-radius: 10px;
    color: white;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    margin-top: 4px;
  }

  .unlock-btn:hover:not(:disabled) {
    background: #d64545;
    transform: scale(1.02);
    box-shadow: 0 6px 20px -6px rgba(197, 61, 61, 0.5);
  }

  .unlock-btn:active:not(:disabled) {
    transform: scale(0.98);
  }

  .unlock-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .security-note {
    margin-top: 20px;
    font-size: 11px;
    color: #48484a;
    line-height: 1.5;
    animation: note-fade 0.8s ease-out 0.8s both;
  }

  @keyframes note-fade {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
