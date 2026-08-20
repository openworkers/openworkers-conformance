// New: the backpressure probe. Production is pull-driven and every chunk
// carries the guest clock at enqueue time, so comparing the guest's spread
// against the host's arrival spread says whether a slow reader actually
// reaches back to the producer.
addEventListener('fetch', function (event) {
    const encoder = new TextEncoder();
    let sent = 0;

    const stream = new ReadableStream({
        pull(controller) {
            if (sent >= {CHUNKS}) {
                controller.close();

                return;
            }

            controller.enqueue(encoder.encode(sent + ':' + Date.now() + '\n'));
            sent += 1;
        }
    });

    event.respondWith(new Response(stream));
});
