// WinterTC (TC55) Minimum Common API: the global surface every runtime must expose.

function assertGlobal(name, kind) {
    const value = globalThis[name];

    assert(typeof value !== 'undefined', name + ' is missing');
    assertType(value, kind || 'function', name);
}

test('AbortController is exposed', () => assertGlobal('AbortController'));

test('AbortSignal is exposed', () => assertGlobal('AbortSignal'));

test('Event is exposed', () => assertGlobal('Event'));

test('EventTarget is exposed', () => assertGlobal('EventTarget'));

test('CustomEvent is exposed', () => assertGlobal('CustomEvent'));

test('ErrorEvent is exposed', () => assertGlobal('ErrorEvent'));

test('MessageChannel is exposed', () => assertGlobal('MessageChannel'));

test('MessageEvent is exposed', () => assertGlobal('MessageEvent'));

test('MessagePort is exposed', () => assertGlobal('MessagePort'));

test('PromiseRejectionEvent is exposed', () => assertGlobal('PromiseRejectionEvent'));

test('atob is exposed', () => assertGlobal('atob'));

test('btoa is exposed', () => assertGlobal('btoa'));

test('setTimeout is exposed', () => assertGlobal('setTimeout'));

test('clearTimeout is exposed', () => assertGlobal('clearTimeout'));

test('setInterval is exposed', () => assertGlobal('setInterval'));

test('clearInterval is exposed', () => assertGlobal('clearInterval'));

test('queueMicrotask is exposed', () => assertGlobal('queueMicrotask'));

test('reportError is exposed', () => assertGlobal('reportError'));

test('structuredClone is exposed', () => assertGlobal('structuredClone'));

test('self is exposed', () => assertGlobal('self', 'object'));

test('navigator.userAgent is exposed', () => {
    assertGlobal('navigator', 'object');
    assertType(navigator.userAgent, 'string');
});

test('the worker error handler attributes are exposed', () => {
    assert('onerror' in globalThis, 'onerror is missing');
    assert('onunhandledrejection' in globalThis, 'onunhandledrejection is missing');
    assert('onrejectionhandled' in globalThis, 'onrejectionhandled is missing');
});

test('DOMException is exposed', () => assertGlobal('DOMException'));

test('Headers is exposed', () => assertGlobal('Headers'));

test('Request is exposed', () => assertGlobal('Request'));

test('Response is exposed', () => assertGlobal('Response'));

test('fetch is exposed', () => assertGlobal('fetch'));

test('FormData is exposed', () => assertGlobal('FormData'));

test('Blob is exposed', () => assertGlobal('Blob'));

test('File is exposed', () => assertGlobal('File'));

test('CompressionStream is exposed', () => assertGlobal('CompressionStream'));

test('DecompressionStream is exposed', () => assertGlobal('DecompressionStream'));

test('ByteLengthQueuingStrategy is exposed', () => assertGlobal('ByteLengthQueuingStrategy'));

test('CountQueuingStrategy is exposed', () => assertGlobal('CountQueuingStrategy'));

test('ReadableStream is exposed', () => assertGlobal('ReadableStream'));

test('ReadableByteStreamController is exposed', () => assertGlobal('ReadableByteStreamController'));

test('ReadableStreamBYOBReader is exposed', () => assertGlobal('ReadableStreamBYOBReader'));

test('ReadableStreamBYOBRequest is exposed', () => assertGlobal('ReadableStreamBYOBRequest'));

test('ReadableStreamDefaultController is exposed', () => assertGlobal('ReadableStreamDefaultController'));

test('ReadableStreamDefaultReader is exposed', () => assertGlobal('ReadableStreamDefaultReader'));

test('TransformStream is exposed', () => assertGlobal('TransformStream'));

test('TransformStreamDefaultController is exposed', () => assertGlobal('TransformStreamDefaultController'));

test('WritableStream is exposed', () => assertGlobal('WritableStream'));

test('WritableStreamDefaultController is exposed', () => assertGlobal('WritableStreamDefaultController'));

test('WritableStreamDefaultWriter is exposed', () => assertGlobal('WritableStreamDefaultWriter'));

test('TextDecoder is exposed', () => assertGlobal('TextDecoder'));

test('TextDecoderStream is exposed', () => assertGlobal('TextDecoderStream'));

test('TextEncoder is exposed', () => assertGlobal('TextEncoder'));

test('TextEncoderStream is exposed', () => assertGlobal('TextEncoderStream'));

test('URL is exposed', () => assertGlobal('URL'));

test('URLSearchParams is exposed', () => assertGlobal('URLSearchParams'));

test('URLPattern is exposed', () => assertGlobal('URLPattern'));

test('crypto is exposed', () => assertGlobal('crypto', 'object'));

test('Crypto is exposed', () => assertGlobal('Crypto'));

test('CryptoKey is exposed', () => assertGlobal('CryptoKey'));

test('SubtleCrypto is exposed', () => assertGlobal('SubtleCrypto'));

test('performance is exposed', () => assertGlobal('performance', 'object'));

test('Performance is exposed', () => assertGlobal('Performance'));

test('WebAssembly is exposed', () => assertGlobal('WebAssembly', 'object'));

test('the WebAssembly object model is exposed', () => {
    assertType(WebAssembly.Module, 'function');
    assertType(WebAssembly.Instance, 'function');
    assertType(WebAssembly.Memory, 'function');
    assertType(WebAssembly.Table, 'function');
    assertType(WebAssembly.Global, 'function');
});

test('the WebAssembly exception model is exposed', () => {
    assertType(WebAssembly.Tag, 'function');
    assertType(WebAssembly.Exception, 'function');
    assertType(WebAssembly.JSTag, 'object');
});

test('the WebAssembly error types are exposed', () => {
    assertType(WebAssembly.CompileError, 'function');
    assertType(WebAssembly.LinkError, 'function');
    assertType(WebAssembly.RuntimeError, 'function');
});

test('the WebAssembly entry points are exposed', () => {
    assertType(WebAssembly.compile, 'function');
    assertType(WebAssembly.instantiate, 'function');
    assertType(WebAssembly.validate, 'function');
});

test('the WebAssembly streaming entry points are exposed', () => {
    assertType(WebAssembly.compileStreaming, 'function');
    assertType(WebAssembly.instantiateStreaming, 'function');
});

test('console is exposed', () => assertGlobal('console', 'object'));

test('globalThis is exposed', () => assertGlobal('globalThis', 'object'));
