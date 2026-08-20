// Ported from openworkers-runtime-v8 test_request_body_stream_echo: the guest
// hands the request body straight to the response and touches no bytes.
addEventListener('fetch', function (event) {
    event.respondWith(new Response(event.request.body));
});
