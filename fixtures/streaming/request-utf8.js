// Ported from openworkers-runtime-v8 test_request_body_stream_utf8_boundary:
// the four bytes of one emoji are split across two chunks.
addEventListener('fetch', async function (event) {
    const text = await event.request.text();
    const chars = Array.from(text).length;

    event.respondWith(new Response('chars=' + chars + ':' + text));
});
