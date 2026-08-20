// Ported from openworkers-runtime-v8 test_request_body_stream_json: the JSON
// arrives split mid-value, so a reassembly bug shows up as a parse failure.
addEventListener('fetch', async function (event) {
    const data = await event.request.json();

    event.respondWith(new Response('name=' + data.name + ',age=' + data.age));
});
