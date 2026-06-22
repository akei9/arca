<script lang="ts">
  import { Icon } from '../icons';
  import { Button, Tag } from '../primitives';
</script>

<section class="page shared-panel" aria-labelledby="shared-title">
  <div class="page__head">
    <span class="page__hash mono">#</span>
    <h1 id="shared-title" class="page__title">shared<em>.</em></h1>
    <div class="page__meta mono">
      <span>scope · <b>local_only</b></span>
      <span>status · <b>deferred</b></span>
    </div>
  </div>

  <div class="shared">
    <div class="shared-deferred" role="status">
      <div class="shared-deferred__mark" aria-hidden="true">
        <Icon name="share" size={18} />
      </div>
      <div>
        <div class="shared-deferred__k mono">sharing_unavailable</div>
        <p>
          shared vaults, secure send, and received secrets are outside this local-first release.
        </p>
      </div>
      <div class="shared-deferred__tags">
        <Tag value="local_only" />
        <Tag value="no_transport" variant="slate" />
        <Tag value="no_export" variant="out" />
      </div>
    </div>

    <div class="shared__grid" aria-label="Deferred sharing capabilities">
      <div class="shared-card">
        <span class="shared-card__k mono">outgoing</span>
        <b>disabled</b>
        <small>no send channel</small>
      </div>
      <div class="shared-card">
        <span class="shared-card__k mono">incoming</span>
        <b>disabled</b>
        <small>no import contract</small>
      </div>
      <div class="shared-card">
        <span class="shared-card__k mono">sync</span>
        <b>offline</b>
        <small>vault stays local</small>
      </div>
    </div>

    <div class="shared__actions" aria-label="Unavailable sharing actions">
      <Button variant="ghost" disabled>
        <Icon name="share" size={12} />
        secure_share
      </Button>
      <Button variant="ghost" disabled>
        <Icon name="external" size={12} />
        import_share
      </Button>
      <p class="shared__note mono">deferred · tracked outside vault runtime</p>
    </div>
  </div>
</section>
