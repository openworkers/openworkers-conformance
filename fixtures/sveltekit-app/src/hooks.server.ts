import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
  event.locals.rid = event.request.headers.get('x-request-id') ?? 'none';

  const response = await resolve(event);

  response.headers.set('x-fixture-hook', event.locals.rid);

  return response;
};
