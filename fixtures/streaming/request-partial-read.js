// Ported from openworkers-runtime-v8 test_request_body_stream_partial_read:
// the guest takes one chunk and cancels, so the producer should see the
// channel close instead of blocking on a full buffer.
addEventListener('fetch', async function (event) {
    const reader = event.request.body.getReader();
    const step = await reader.read();
    const first = new TextDecoder().decode(step.value);

    await reader.cancel();

    event.respondWith(new Response('first=' + first));
});
