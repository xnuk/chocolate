// oxlint-disable no-console

import { serve, file } from 'bun'
import web from './src/web/index.html'

const server = serve({
	port: 8080,
	routes: {
		'/': web,
		'/chocolate_bg.wasm': {
			async GET(_req) {
				const wasm = await file(
					'./build/wasm/chocolate_bg.wasm',
				).arrayBuffer()

				return new Response(wasm, {
					headers: { 'content-type': 'application/wasm' },
				})
			},
		},
	},
	development: { hmr: true, console: true },
})

console.log(`Server running at http://${server.hostname}:${server.port}`)
