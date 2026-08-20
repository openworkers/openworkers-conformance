// Ported from openworkers-runtime-v8 test_request_body_stream_never_consumed:
// the guest answers without reading, and the producer must not be stranded.
addEventListener('fetch', function (event) {
    event.respondWith(new Response('ignored'));
});
