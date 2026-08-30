export default defineEventHandler((event) => {
  const path = event.path.split('?')[0] ?? ''
  if (path === '/_nuxt' || path === '/_nuxt/') {
    throw createError({ statusCode: 404, statusMessage: 'Not Found' })
  }
})
