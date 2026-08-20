// Ported from openworkers-runtime-v8 test_request_body_stream_text.
addEventListener('fetch', async function (event) {
    const text = await event.request.text();

    event.respondWith(new Response('Got: ' + text));
});
