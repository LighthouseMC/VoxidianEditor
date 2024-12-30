const C2S_HANDSHAKE = 0;


let session_code = window.location.hash.slice(1);
window.history.replaceState("", "", window.location.pathname);

var socket_protocol = "ws:";
if (location.protocol === "https:") {
    socket_protocol = "wss:";
}
const SOCKET = new WebSocket(socket_protocol + "//" + window.location.hostname + ":" + window.location.port + "/editor/ws", "{{VOXIDIAN_EDITOR_NAME}}");
// TODO: Change to wss, or auto detect.
var socket_queued_data = [];
SOCKET.addEventListener("close", (event) => {
    // TODO: Disconnected popup
});

SOCKET.addEventListener("open", (_) => {
    send_c2s_order(C2S_HANDSHAKE, new TextEncoder().encode(session_code));
});

SOCKET.addEventListener("message", (event) => {
    let reader = new FileReader();
    reader.onload = (event) => {
        let prefixed_data = new Uint8Array(event.target.result);
        handle_s2c_order(prefixed_data[0], prefixed_data.slice(1));
    };
    reader.readAsArrayBuffer(event.data);
});

function send_c2s_order(prefix, data) {
    let final_data = new Uint8Array(data.length + 1);
    final_data.set(prefix);
    final_data.set(data, 1);
    let buffer = final_data.buffer.slice(final_data.byteOffset, final_data.byteLength + final_data.byteOffset);
    SOCKET.send(buffer);
}

function handle_s2c_order(prefix, data) {
    console.warn(prefix);
    console.warn(data);
}


// TODO: Show loading overlay until ready.
