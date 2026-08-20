// Ported from openworkers-runtime-v8 test_request_body_stream_arraybuffer and
// test_request_body_stream_binary_nulls: byte-exact, embedded NULs included.
addEventListener('fetch', async function (event) {
    const buffer = await event.request.arrayBuffer();
    const bytes = new Uint8Array(buffer);
    let sum = 0;
    let nulls = 0;

    for (let i = 0; i < bytes.length; i++) {
        sum += bytes[i];

        if (bytes[i] === 0) {
            nulls += 1;
        }
    }

    event.respondWith(new Response('len=' + bytes.length + ',sum=' + sum + ',nulls=' + nulls));
});
