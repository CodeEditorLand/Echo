var s = async (...[e = null, t = 200]) =>
	new a(JSON.stringify(e), {
		status: t,
		headers: { "Content-Type": "application/json;charset=utf-8" },
	});
const { Response: a } = await import(
	"@cloudflare/workers-types/experimental/index.js"
);
export { a as Response, s as default };
