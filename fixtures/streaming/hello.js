// The control: no streams at all. Used to check a heap cap leaves room for the
// baseline, and to check a worker still answers after a cancelled stream.
addEventListener('fetch', function (event) {
    event.respondWith(new Response('hello'));
});
