/**
 * @module Worker
 *
 */
declare const _default: {
    fetch: (Request: import("@cloudflare/workers-types/experimental/index.js").Request<unknown, import("@cloudflare/workers-types/experimental/index.js").CfProperties<unknown>>, Environment: import("../Interface/Environment.js").default, Context: import("@cloudflare/workers-types/experimental/index.js").ExecutionContext) => Response;
};
export default _default;
export declare const Access: import("@codeeditorland/common/Target/Interface/Access.js").default;
export declare const Put: import("@codeeditorland/common/Target/Interface/Put.js").default;
export declare const WebSocketPair: new () => {
    0: import("@cloudflare/workers-types/experimental/index.js").WebSocket;
    1: import("@cloudflare/workers-types/experimental/index.js").WebSocket;
};
