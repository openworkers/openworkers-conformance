// Ported from openworkers-runtime-v8 test_request_body_stream_error: the host
// pushes Err(...) into the request channel mid-body. A clean read error is the
// wanted outcome; silent truncation is the interesting failure.
addEventListener('fetch', async function (event) {
    const reader = event.request.body.getReader();
    const decoder = new TextDecoder();
    const chunks = [];

    try {
        while (true) {
            const step = await reader.read();

            if (step.done) {
                event.respondWith(new Response('done:' + chunks.join('|')));

                return;
            }

            chunks.push(decoder.decode(step.value));
        }
    } catch (e) {
        const message = (e && e.message) || String(e);

        event.respondWith(new Response('threw:' + chunks.join('|') + ':' + message));
    }
});
