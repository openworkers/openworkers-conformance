// Ported from openworkers-runtime-v8 test_request_body_stream_large: how many
// chunks the guest sees is itself a measurement, so it is reported alongside
// the byte count.
addEventListener('fetch', async function (event) {
    const reader = event.request.body.getReader();
    let chunks = 0;
    let bytes = 0;

    while (true) {
        const step = await reader.read();

        if (step.done) {
            break;
        }

        chunks += 1;
        bytes += step.value.length;
    }

    event.respondWith(new Response('chunks=' + chunks + ',bytes=' + bytes));
});
