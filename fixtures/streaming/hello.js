// The control: no streams at all. Also used to check that a heap cap leaves
// room for the baseline before a probe blames streaming for a boot failure.
addEventListener('fetch', function (event) {
    event.respondWith(new Response('hello'));
});
