// Ported from openworkers-runtime-v8 test_request_body_stream_double_consume:
// the second read has to fail, a body is consumed once.
addEventListener('fetch', async function (event) {
    const first = await event.request.text();

    try {
        await event.request.text();

        event.respondWith(new Response('second read succeeded'));
    } catch (e) {
        event.respondWith(new Response('bodyUsed:' + first));
    }
});
