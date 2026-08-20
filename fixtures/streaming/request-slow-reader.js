// Ported from the openworkers-runner backpressure corpus
// (test_backpressure_input_slow_consumer, test_backpressure_no_data_loss):
// the guest waits {DELAY}ms per chunk, so a fast host producer has to be held
// back by the channel instead of piling up somewhere.
addEventListener('fetch', async function (event) {
    const reader = event.request.body.getReader();
    const decoder = new TextDecoder();
    let chunks = 0;
    let sum = 0;

    while (true) {
        const step = await reader.read();

        if (step.done) {
            break;
        }

        await new Promise(function (resolve) {
            setTimeout(resolve, {DELAY});
        });

        chunks += 1;
        sum += parseInt(decoder.decode(step.value), 10);
    }

    event.respondWith(new Response('chunks=' + chunks + ',sum=' + sum));
});
