// Ported from openworkers-runtime-v8 test_request_body_stream_transform and
// test_request_body_stream_bidirectional: read one number, emit its double,
// one pull at a time, while the request is still arriving.
addEventListener('fetch', function (event) {
    const reader = event.request.body.getReader();
    const decoder = new TextDecoder();
    const encoder = new TextEncoder();

    const stream = new ReadableStream({
        async pull(controller) {
            const step = await reader.read();

            if (step.done) {
                controller.close();

                return;
            }

            const value = parseInt(decoder.decode(step.value), 10);

            controller.enqueue(encoder.encode((value * 2).toString()));
        }
    });

    event.respondWith(new Response(stream));
});
