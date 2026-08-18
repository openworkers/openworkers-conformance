<script lang="ts">
  let { data } = $props();
</script>

<p id="fixed">{data.fixed}</p>
<p id="supplied">{data.supplied}</p>
<p id="attr" title={data.attr}>attr</p>
<p id="raw">{@html '<em>raw</em>'}</p>
