// New: a long paced stream for the cancellation probe. The host walks away
// after a few chunks; what the guest does next is the measurement.
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

                try {
                    controller.enqueue(encoder.encode(sent + ':' + Date.now() + '\n'));
                } catch (e) {
                    return;
                }

                sent += 1;

                setTimeout(step, {DELAY});
            }

            setTimeout(step, {DELAY});
        }
    });

    event.respondWith(new Response(stream));
});
