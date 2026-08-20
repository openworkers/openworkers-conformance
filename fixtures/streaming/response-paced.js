// New: the time-to-first-chunk probe. The guest emits {CHUNKS} chunks {DELAY}ms
// apart and stamps each one with its own clock, so the host can tell a stream
// that flows from one that only arrives once the guest is done.
addEventListener('fetch', function (event) {
    const encoder = new TextEncoder();
    let sent = 0;

    const stream = new ReadableStream({
        start(controller) {
            function step() {
                if (sent >= {CHUNKS}) {
                    controller.close();

                    return;
                }

                controller.enqueue(encoder.encode(sent + ':' + Date.now() + '\n'));
                sent += 1;

                setTimeout(step, {DELAY});
            }

            setTimeout(step, {DELAY});
        }
    });

    event.respondWith(new Response(stream));
});
