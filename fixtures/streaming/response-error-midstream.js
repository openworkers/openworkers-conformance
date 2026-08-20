// The guest errors its own stream after {GOOD} chunks. The host should
// see an Err(...) on the channel, not a body that just stops.
addEventListener('fetch', function (event) {
    const encoder = new TextEncoder();
    let sent = 0;

    const stream = new ReadableStream({
        pull(controller) {
            if (sent >= {GOOD}) {
                controller.error(new Error('guest gave up mid-stream'));

                return;
            }

            controller.enqueue(encoder.encode('chunk' + sent));
            sent += 1;
        }
    });

    event.respondWith(new Response(stream));
});
