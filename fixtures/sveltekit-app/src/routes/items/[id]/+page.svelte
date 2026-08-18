<script lang="ts">
  let { data, form } = $props();

  const pairs = [...data.exotic.map].map(([k, v]) => `${k}=${v}`).join(' ');
</script>

<h1>{data.item.name}</h1>

<dl>
  <dt>id</dt>
  <dd id="id">{data.item.id}</dd>
  <dt>price</dt>
  <dd id="price">{data.item.price}</dd>
  <dt>visits</dt>
  <dd id="visits">{data.visits}</dd>
  <dt>query</dt>
  <dd id="query">{data.query ?? ''}</dd>
  <dt>tags</dt>
  <dd id="tags">{data.tags.join(',')}</dd>
  <dt>accept</dt>
  <dd id="accept">{data.accept ?? ''}</dd>
  <dt>date</dt>
  <dd id="date">{data.exotic.date.toISOString()}</dd>
  <dt>map</dt>
  <dd id="map">{pairs}</dd>
  <dt>set</dt>
  <dd id="set">{[...data.exotic.set].join('+')}</dd>
  <dt>big</dt>
  <dd id="big">{data.exotic.big}</dd>
  <dt>unicode</dt>
  <dd id="unicode">{data.exotic.unicode}</dd>
  <dt>regexp</dt>
  <dd id="regexp">{data.exotic.re.source}/{data.exotic.re.flags}</dd>
  <dt>shared</dt>
  <dd id="shared">{data.exotic.ref1 === data.exotic.ref2}</dd>
</dl>

<form method="POST" action="?/greet">
  <input name="name" value="" />
  <button>greet</button>
</form>

{#if form}
  <p id="form">{form.greeting ?? form.message}</p>
{/if}
