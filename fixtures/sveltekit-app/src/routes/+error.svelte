<script lang="ts">
  import { page } from '$app/state';
</script>

<h1>Error {page.status}</h1>
<p id="message">{page.error?.message}</p>
