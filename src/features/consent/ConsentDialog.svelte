<script lang="ts">
  import Dialog from "../../lib/components/Dialog.svelte";
  import Button from "../../lib/components/Button.svelte";
  import VerificationCode from "../../lib/components/VerificationCode.svelte";
  import { t } from "../../lib/i18n";
  import type { ConsentModel } from "./model.svelte";

  interface Props {
    model: ConsentModel;
  }

  let { model }: Props = $props();
</script>

{#if model.current}
  {@const item = model.current}
  <Dialog
    title={item.kind === "session"
      ? t("peers.incoming.sessionTitle")
      : t("peers.incoming.pairingTitle")}
    onClose={() => void model.respond(false)}
  >
    <div class="nb-consent-content">
      <p class="nb-consent-desc">
        {t(item.kind === "session" ? "peers.incoming.sessionDesc" : "peers.incoming.pairingDesc", {
          peer: item.request.peer.displayName,
        })}
      </p>

      {#if item.kind === "session"}
        <dl class="nb-consent-plan">
          <dt>{t("plan.protocol")}</dt>
          <dd>{item.request.plan.protocol.toUpperCase()}</dd>
          <dt>{t("plan.streams")}</dt>
          <dd>{item.request.plan.streams}</dd>
        </dl>
      {:else}
        <div class="nb-consent-code">
          <VerificationCode code={item.request.verificationCode} />
        </div>
      {/if}

      {#if model.error}
        <p class="nb-consent-error" role="alert">{model.error}</p>
      {/if}

      <div class="nb-dialog-actions">
        <Button
          variant="ghost"
          disabled={model.busy}
          onclick={() => void model.respond(false)}
          data-testid="consent-reject-btn"
        >
          {t("peers.incoming.reject")}
        </Button>
        <Button
          variant="primary"
          disabled={model.busy}
          onclick={() => void model.respond(true)}
          data-testid="consent-accept-btn"
        >
          {t("peers.incoming.accept")}
        </Button>
      </div>
    </div>
  </Dialog>
{/if}

<style>
  .nb-consent-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .nb-consent-desc {
    font-size: var(--font-size-sm);
    color: var(--color-text-secondary);
    margin: 0;
  }

  .nb-consent-plan {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--space-1) var(--space-3);
    font-size: var(--font-size-sm);
    margin: 0;
  }

  .nb-consent-plan dt {
    color: var(--color-text-muted);
  }

  .nb-consent-plan dd {
    margin: 0;
    color: var(--color-text-primary);
    font-weight: var(--font-weight-title);
  }

  .nb-consent-code {
    display: flex;
    justify-content: center;
    padding: var(--space-4) 0;
  }

  .nb-consent-error {
    color: var(--color-danger);
    font-size: var(--font-size-sm);
    margin: 0;
  }

  .nb-dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
  }
</style>
