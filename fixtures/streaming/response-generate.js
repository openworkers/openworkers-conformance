// {CHUNKS} chunks of {CHUNK_SIZE} bytes, produced as fast as the runtime
// takes them. The host hashes what comes out, so a dropped or reordered chunk
// cannot pass as a size match.
addEventListener('fetch', function (event) {
    const chunk = new Uint8Array({CHUNK_SIZE});

    for (let i = 0; i < chunk.length; i++) {
        chunk[i] = i & 0xff;
    }

    let sent = 0;

    const stream = new ReadableStream({
        pull(controller) {
            if (sent >= {CHUNKS}) {
                controller.close();

                return;
            }

            controller.enqueue(chunk.slice(0));
            sent += 1;
        }
    });

    event.respondWith(new Response(stream));
});
