const e_template_subserver_id                   = document.getElementsByClassName("template_subserver_id");
const e_template_subserver_name                 = document.getElementsByClassName("template_subserver_name");
const e_subserver_properties_editor_name        = document.getElementById("subserver_properties_editor_name");
const e_template_subserver_description          = document.getElementsByClassName("template_subserver_description");
const e_subserver_properties_editor_description = document.getElementById("subserver_properties_editor_description");
const e_template_subserver_owner_name           = document.getElementsByClassName("template_subserver_owner_name");

const e_editor_filetree_root = document.getElementById("editor_filetree_root");
let filetree_path_element = [{ is_root : true, file_path : "", children : e_editor_filetree_root }];


const C2S_HANDSHAKE = 0;
const C2S_KEEPALIVE = 1;

const S2C_INITIAL_STATE = 0;
const S2C_KEEPALIVE     = 1;

let keepalive_ping_index = 0;


function MessageBuf(bytes) {
    let buf = {
        inner : bytes,
        head : 0
    };
    buf.read = function(count) {
        let slice = buf.inner.slice(buf.head, buf.head + count);
        buf.head += count;
        return slice;
    }
    buf.read_str = function() {
        let str_len = u32ToInt(buf.read(4));
        let slice = buf.inner.slice(buf.head, buf.head + str_len);
        buf.head += str_len;
        return new TextDecoder("utf-8").decode(slice);
    }
    return buf;
}


let session_code = window.location.hash.slice(1);
window.history.replaceState("", "", window.location.pathname);

let socket_protocol = "ws:";
if (location.protocol === "https:") {
    socket_protocol = "wss:";
}
const SOCKET = new WebSocket(socket_protocol + "//" + window.location.hostname + ":" + window.location.port + "/editor/ws", "{{VOXIDIAN_EDITOR_NAME}}");
// TODO: Change to wss, or auto detect.
let socket_queued_data = [];
SOCKET.addEventListener("close", (event) => {
    // TODO: Disconnected popup
});

SOCKET.addEventListener("open", (_) => {
    send_c2s_order(C2S_HANDSHAKE, new TextEncoder("utf-8").encode(session_code));
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
    final_data.set([prefix], 0);
    final_data.set(data, 1);
    let buffer = final_data.buffer.slice(final_data.byteOffset, final_data.byteLength + final_data.byteOffset);
    SOCKET.send(buffer);
}

function handle_s2c_order(prefix, data) {


    if (prefix == S2C_INITIAL_STATE) {
        let buf = MessageBuf(data);
        let subserver_id          = u32ToInt(buf.read(4));
        let subserver_name        = buf.read_str();
        let subserver_description = buf.read_str();
        let subserver_owner_name  = buf.read_str();
        let file_entity_count = u32ToInt(buf.read(4));
        for (let i = 0; i < file_entity_count; i++) {
            let file_id     = u32ToInt(buf.read(4));
            let is_dir      = buf.read(1)[0] != 0;
            let file_path   = buf.read_str();
            let file_name   = file_path.split("/").slice(-1)[0];
            let element     = document.createElement("li");
            let children;
            let data = {
                file_path           : file_path,
                file_path_lowercase : file_path.toLowerCase(),
                is_dir              : is_dir,
                main                : element,
            };
            if (is_dir) {
                let fold = document.createElement("div");
                fold.classList = "editor_filetree_fold";
                fold.innerText = file_name;
                element.appendChild(fold);
                let nest = document.createElement("ul");
                nest.classList = "editor_filetree_nest";
                element.appendChild(nest);
                fold.addEventListener("click", function() {
                    fold.classList.toggle("editor_filetree_unfolded");
                    nest.classList.toggle("editor_filetree_nest_unfolded");
                });
                data.children = nest;
            } else {
                element.innerHTML = "<div><i class=\"" + fnameToIcon(file_name) + "\"></i> " + file_name + "</div>";
                data.children = element;
            }
            filetree_path_element.push(data);
        }
        filetree_path_element = filetree_path_element.sort((a, b) => {
            if (a.is_dir) {
                if (b.is_dir) {
                    if      (a.file_path_lowercase > b.file_path_lowercase) { return 1; }
                    else if (a.file_path_lowercase < b.file_path_lowercase) { return -1; }
                    else {
                        if (a.file_path > b.file_path) { return 1; }
                        if (a.file_path < b.file_path) { return -1; }
                        else { console.error("unreachable"); }
                    }
                }
                else { return -1; }
            } else {
                if (b.is_dir) { return 1; }
                else {
                    if      (a.file_path_lowercase > b.file_path_lowercase) { return 1; }
                    else if (a.file_path_lowercase < b.file_path_lowercase) { return -1; }
                    else {
                        if (a.file_path > b.file_path) { return 1; }
                        if (a.file_path < b.file_path) { return -1; }
                        else { console.error("unreachable"); }
                    }
                }
            }
        });

        for (let i = 0; i < e_template_subserver_id.length; i++) {
            e_template_subserver_id[i].innerText = "" + subserver_id;
        }
        for (let i = 0; i < e_template_subserver_name.length; i++) {
            e_template_subserver_name[i].innerText = subserver_name;
        }
        e_subserver_properties_editor_name.value = subserver_name;
        for (let i = 0; i < e_template_subserver_description.length; i++) {
            e_template_subserver_description[i].innerText = subserver_description;
        }
        e_subserver_properties_editor_description.value = subserver_description;
        for (let i = 0; i < e_template_subserver_owner_name.length; i++) {
            e_template_subserver_owner_name[i].innerText = subserver_owner_name;
        }

        for (let i = 0; i < filetree_path_element.length; i++) {
            let entry = filetree_path_element[i];
            if (entry.is_root !== true) {
                let parent_path = entry.file_path.split("/").slice(0, -1).join("/");
                let parent      = filetree_path_element.find((e) => e.file_path == parent_path);
                parent.children.appendChild(entry.main);
            }
        }
    }


    else if (prefix == S2C_KEEPALIVE) {
        send_c2s_order(C2S_KEEPALIVE, intToU32(keepalive_ping_index));
        keepalive_ping_index = (keepalive_ping_index + 1) % 4294967295;
    }


}


function intToU32(i) {
    return Uint8Array.of(
        (i & 0xff000000) >> 24,
        (i & 0x00ff0000) >> 16,
        (i & 0x0000ff00) >>  8,
        (i & 0x000000ff)
    );
}
function u32ToInt(u) {
    return (u[0] << 24) | (u[1] << 16) | (u[2] << 8) | u[3];
}
function fnameToIcon(fname) {
    let parts = fname.split(".");
    if (parts.length > 1) {
        let ext = parts[parts.length - 1];
        switch (ext) {

            // Bash
            case "sh" : return "devicon-bash-plain icon";

            // C
            case "c"    : return "devicon-c-plain colored icon";
            case "cats" : return "devicon-c-plain colored icon";
            case "h"    : return "devicon-c-plain colored icon";
            case "idc"  : return "devicon-c-plain colored icon";
            case "w"    : return "devicon-c-plain colored icon";

            // C++
            case "cpp" : return "devicon-cplusplus-plain colored icon";
            case "cc"  : return "devicon-cplusplus-plain colored icon";
            case "cp"  : return "devicon-cplusplus-plain colored icon";
            case "cxx" : return "devicon-cplusplus-plain colored icon";
            case "hh"  : return "devicon-cplusplus-plain colored icon";
            case "hpp" : return "devicon-cplusplus-plain colored icon";
            case "hxx" : return "devicon-cplusplus-plain colored icon";
            case "inc" : return "devicon-cplusplus-plain colored icon";
            case "inl" : return "devicon-cplusplus-plain colored icon";
            case "ipp" : return "devicon-cplusplus-plain colored icon";
            case "tcc" : return "devicon-cplusplus-plain colored icon";
            case "tpp" : return "devicon-cplusplus-plain colored icon";

            // Rust
            case "rs" : return "devicon-rust-original icon";

            // Go
            case "go" : return "devicon-go-original-wordmark colored icon";

            // C#
            case "cs"     : return "devicon-csharp-plain colored icon";
            case "cake"   : return "devicon-csharp-plain colored icon";
            case "cshtml" : return "devicon-csharp-plain colored icon";
            case "csx"    : return "devicon-csharp-plain colored icon";

            // Dart
            case "dart" : return "devicon-dart-plain colored icon";

            // F#
            case "fs"  : return "devicon-fsharp-plain colored icon";
            case "fsi" : return "devicon-fsharp-plain colored icon";
            case "fsx" : return "devicon-fsharp-plain colored icon";

            // LabVIEW
            case "lvproj" : return "devicon-labview-plain colored icon";

            // Lua
            case "lua"    : return "devicon-lua-plain colored icon";
            case "fcgi"   : return "devicon-lua-plain colored icon";
            case "nse"    : return "devicon-lua-plain colored icon";
            case "pd_lua" : return "devicon-lua-plain colored icon";
            case "rbxs"   : return "devicon-lua-plain colored icon";
            case "wlua"   : return "devicon-lua-plain colored icon";

            // TypeScript
            case "ts"  : return "devicon-typescript-plain colored icon";
            case "tsx" : return "devicon-typescript-plain colored icon";

            // WebAssembly
            case "wasm" : return "devicon-wasm-original colored icon";
            case "wat"  : return "devicon-wasm-original colored icon";

            // Zig
            case "zig" : return "devicon-zig-original colored icon";

            // Crystal
            case "cr" : return "devicon-crystal-original icon";

            // Elixir
            case "ex"  : return "devicon-elixir-plain colored icon";
            case "exs" : return "devicon-elixir-plain colored icon";

            // Java
            case "java"  : return "devicon-java-plain colored icon";
            case "class" : return "devicon-java-plain colored icon";

            // Javascript
            case "js"                  : return "devicon-javascript-plain colored icon";
            case "_js"                 : return "devicon-javascript-plain colored icon";
            case "bones"               : return "devicon-javascript-plain colored icon";
            case "es"                  : return "devicon-javascript-plain colored icon";
            case "es6"                 : return "devicon-javascript-plain colored icon";
            case "frag"                : return "devicon-javascript-plain colored icon";
            case "gs"                  : return "devicon-javascript-plain colored icon";
            case "jake"                : return "devicon-javascript-plain colored icon";
            case "jsb"                 : return "devicon-javascript-plain colored icon";
            case "jscad"               : return "devicon-javascript-plain colored icon";
            case "jsfl"                : return "devicon-javascript-plain colored icon";
            case "jsm"                 : return "devicon-javascript-plain colored icon";
            case "jss"                 : return "devicon-javascript-plain colored icon";
            case "njs"                 : return "devicon-javascript-plain colored icon";
            case "pac"                 : return "devicon-javascript-plain colored icon";
            case "sjs"                 : return "devicon-javascript-plain colored icon";
            case "ssjs"                : return "devicon-javascript-plain colored icon";
            case "sublime-build"       : return "devicon-javascript-plain colored icon";
            case "sublime-commands"    : return "devicon-javascript-plain colored icon";
            case "sublime-completions" : return "devicon-javascript-plain colored icon";
            case "sublime-keymap"      : return "devicon-javascript-plain colored icon";
            case "sublime-macro"       : return "devicon-javascript-plain colored icon";
            case "sublime-menu"        : return "devicon-javascript-plain colored icon";
            case "sublime-mousemap"    : return "devicon-javascript-plain colored icon";
            case "sublime-project"     : return "devicon-javascript-plain colored icon";
            case "sublime-settings"    : return "devicon-javascript-plain colored icon";
            case "sublime-theme"       : return "devicon-javascript-plain colored icon";
            case "sublime-workspace"   : return "devicon-javascript-plain colored icon";
            case "sublime_metrics"     : return "devicon-javascript-plain colored icon";
            case "sublime_session"     : return "devicon-javascript-plain colored icon";
            case "xsjs"                : return "devicon-javascript-plain colored icon";
            case "xsjslib"             : return "devicon-javascript-plain colored icon";

            // Kotlin
            case "kt"  : return "devicon-kotlin-plain colored icon";
            case "ktm" : return "devicon-kotlin-plain colored icon";
            case "kts" : return "devicon-kotlin-plain colored icon";

            // Perl
            case "pl"    : return "devicon-perl-plain colored icon";
            case "al"    : return "devicon-perl-plain colored icon";
            case "perl"  : return "devicon-perl-plain colored icon";
            case "ph"    : return "devicon-perl-plain colored icon";
            case "plx"   : return "devicon-perl-plain colored icon";
            case "pm"    : return "devicon-perl-plain colored icon";
            case "pod"   : return "devicon-perl-plain colored icon";
            case "psgi"  : return "devicon-perl-plain colored icon";
            case "t"     : return "devicon-perl-plain colored icon";
            case "6pl"   : return "devicon-perl-plain colored icon";
            case "6pm"   : return "devicon-perl-plain colored icon";
            case "nqp"   : return "devicon-perl-plain colored icon";
            case "p6"    : return "devicon-perl-plain colored icon";
            case "p6l"   : return "devicon-perl-plain colored icon";
            case "p6m"   : return "devicon-perl-plain colored icon";
            case "pl6"   : return "devicon-perl-plain colored icon";
            case "pm6"   : return "devicon-perl-plain colored icon";

            // PHP
            case "php"  : return "devicon-php-plain colored icon";
            case "aw"   : return "devicon-php-plain colored icon";
            case "ctp"  : return "devicon-php-plain colored icon";
            case "inc"  : return "devicon-php-plain colored icon";
            case "php3" : return "devicon-php-plain colored icon";
            case "php4" : return "devicon-php-plain colored icon";
            case "php5" : return "devicon-php-plain colored icon";
            case "phps" : return "devicon-php-plain colored icon";
            case "phpt" : return "devicon-php-plain colored icon";

            // Python
            case "py"    : return "devicon-python-plain colored icon";
            case "bzl"   : return "devicon-python-plain colored icon";
            case "cgi"   : return "devicon-python-plain colored icon";
            case "fcgi"  : return "devicon-python-plain colored icon";
            case "gyp"   : return "devicon-python-plain colored icon";
            case "lmi"   : return "devicon-python-plain colored icon";
            case "pyde"  : return "devicon-python-plain colored icon";
            case "pyp"   : return "devicon-python-plain colored icon";
            case "pyt"   : return "devicon-python-plain colored icon";
            case "pyw"   : return "devicon-python-plain colored icon";
            case "rpy"   : return "devicon-python-plain colored icon";
            case "tac"   : return "devicon-python-plain colored icon";
            case "wsgi"  : return "devicon-python-plain colored icon";
            case "xpy"   : return "devicon-python-plain colored icon";

            // Prolog
            case "pro"    : return "devicon-prolog-plain colored icon";
            case "prolog" : return "devicon-prolog-plain colored icon";
            case "yap"    : return "devicon-prolog-plain colored icon";

            // R
            case "r"   : return "devicon-r-plain colored icon";
            case "rd"  : return "devicon-r-plain colored icon";
            case "rsx" : return "devicon-r-plain colored icon";

            // Ruby
            case "rb"         : return "devicon-ruby-plain colored icon";
            case "builder"    : return "devicon-ruby-plain colored icon";
            case "fcgi"       : return "devicon-ruby-plain colored icon";
            case "gemspec"    : return "devicon-ruby-plain colored icon";
            case "god"        : return "devicon-ruby-plain colored icon";
            case "irbrc"      : return "devicon-ruby-plain colored icon";
            case "jbuilder"   : return "devicon-ruby-plain colored icon";
            case "mspec"      : return "devicon-ruby-plain colored icon";
            case "pluginspec" : return "devicon-ruby-plain colored icon";
            case "podspec"    : return "devicon-ruby-plain colored icon";
            case "rabl"       : return "devicon-ruby-plain colored icon";
            case "rake"       : return "devicon-ruby-plain colored icon";
            case "rbuild"     : return "devicon-ruby-plain colored icon";
            case "rbw"        : return "devicon-ruby-plain colored icon";
            case "rbx"        : return "devicon-ruby-plain colored icon";
            case "ru"         : return "devicon-ruby-plain colored icon";
            case "ruby"       : return "devicon-ruby-plain colored icon";
            case "thor"       : return "devicon-ruby-plain colored icon";
            case "watchr"     : return "devicon-ruby-plain colored icon";

            // Swift
            case "swift" : return "devicon-swift-plain colored icon";

            // Ballerina
            case "bal" : return "devicon-ballerina-original colored icon";

            // Haskell
            case "hs"  : return "devicon-haskell-plain colored icon";
            case "hsc" : return "devicon-haskell-plain colored icon";

            // Julia
            case "hl" : return "devicon-julia-plain colored icon";

            // Nim
            case "nim"    : return "devicon-nim-plain colored icon";
            case "nimrod" : return "devicon-nim-plain colored icon";

            // OCaml
            case "ml"     : return "devicon-ocaml-plain colored icon";
            case "eliom"  : return "devicon-ocaml-plain colored icon";
            case "eliomi" : return "devicon-ocaml-plain colored icon";
            case "ml4"    : return "devicon-ocaml-plain colored icon";
            case "mli"    : return "devicon-ocaml-plain colored icon";
            case "mll"    : return "devicon-ocaml-plain colored icon";
            case "mly"    : return "devicon-ocaml-plain colored icon";

        }
    }
    return "devicon-bash-plain noicon";
}


// TODO: Show loading overlay until ready.
