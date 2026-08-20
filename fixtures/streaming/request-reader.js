// Ported from openworkers-runtime-v8 test_request_body_stream_reader.
addEventListener('fetch', async function (event) {
    const reader = event.request.body.getReader();
    const decoder = new TextDecoder();
    const chunks = [];

    while (true) {
        const step = await reader.read();

        if (step.done) {
            break;
        }

        chunks.push(decoder.decode(step.value));
    }

    event.respondWith(new Response('chunks=' + chunks.length + ':' + chunks.join('|')));
});
