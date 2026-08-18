import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = ({ url, request, locals }) => {
  if (url.searchParams.get('boom') === '1') {
    error(418, 'teapot');
  }

  return json(
    {
      method: 'GET',
      path: url.pathname,
      q: url.searchParams.getAll('q'),
      accept: request.headers.get('accept'),
      rid: locals.rid
    },
    { headers: { 'x-fixture': 'echo' } }
  );
};

export const POST: RequestHandler = async ({ request, locals }) => {
  const body = await request.json();

  return json({
    method: 'POST',
    keys: Object.keys(body).sort(),
    received: body,
    rid: locals.rid
  });
};
